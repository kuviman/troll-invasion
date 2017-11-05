#[cfg(not(target_os = "emscripten"))]
extern crate ws;
#[cfg(not(target_os = "emscripten"))]
extern crate env_logger;
extern crate argparse;
#[macro_use]
extern crate lazy_static;
extern crate conrod;
extern crate rusttype;

#[macro_use]
extern crate codevisual;
#[cfg(target_os = "emscripten")]
#[macro_use]
extern crate web;

#[cfg(target_os = "emscripten")]
extern crate troll_invasion_web;

pub ( crate ) use codevisual::prelude::*;
pub ( crate ) use codevisual::ugli;

#[cfg(not(target_os = "emscripten"))]
mod server;
mod client;
#[cfg(not(target_os = "emscripten"))]
mod console_client;
mod ui_renderer;

pub ( crate ) use ui_renderer::UiRenderer;

enum ServerMessage {
    ReadyStatus {
        nick: String,
        ready: bool,
    },
    MapLine(usize, Vec<Option<GameCell>>),
    GameStart,
    PlayerColor {
        nick: String,
        color: char,
    },
    Turn {
        nick: String,
    },
    SelectCell {
        row: usize,
        col: usize,
    },
    GameFinish,
    UpgradePhase,
    EnergyLeft(usize),
}

impl ServerMessage {
    fn parse(message: &str) -> Self {
        use ServerMessage::*;
        let mut args = message.split_whitespace();
        let command = args.next().unwrap();
        match command {
            "readyStatus" => ReadyStatus {
                nick: args.next().unwrap().to_owned(),
                ready: args.next().unwrap().parse().unwrap(),
            },
            "gameStart" => GameStart,
            "playerColor" => PlayerColor {
                nick: args.next().unwrap().to_owned(),
                color: args.next().unwrap().parse().unwrap(),
            },
            "turn" => Turn {
                nick: args.next().unwrap().to_owned(),
            },
            "selectCell" => SelectCell {
                row: args.next().unwrap().parse().unwrap(),
                col: args.next().unwrap().parse().unwrap(),
            },
            "gameFinish" => GameFinish,
            "upgradePhase" => UpgradePhase,
            "energyLeft" => EnergyLeft(args.next().unwrap().parse().unwrap()),
            "mapLine" => {
                let index = args.next().unwrap().parse().unwrap();
                let cells = args.next().unwrap().split('|').map(|cell| {
                    match cell {
                        "##" => Some(GameCell::Empty),
                        "__" => None,
                        _ => {
                            let (count, owner) = cell.split_at(cell.len() - 1);
                            let count = count.parse().unwrap();
                            let owner = owner.parse().unwrap();
                            Some(GameCell::Populated {
                                count,
                                owner,
                            })
                        }
                    }
                }).collect();
                MapLine(index, cells)
            }
            _ => panic!("Unexpected message: {:?}", message)
        }
    }
}

#[derive(Copy, Clone)]
enum GameCell {
    Empty,
    Populated {
        count: usize,
        owner: char,
    }
}

lazy_static! {
    static ref HOST: Mutex<String> = Mutex::new(String::new());
    static ref PORT: Mutex<u16> = Mutex::new(0);
    static ref NICK: Mutex<String> = Mutex::new(String::new());
}

fn main() {
    #[cfg(target_os = "emscripten")]
    web::run_script(troll_invasion_web::JS_SOURCE);

    #[cfg(not(target_os = "emscripten"))]
    env_logger::init().unwrap();

    let mut port: u16 = 8008;
    let mut host = None;
    let mut start_server = false;
    let mut console = false;
    let mut nickname = None;

    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("TrollInvasion client/server. By default starts client connecting to play.kuviman.com.");
        ap.refer(&mut port).add_option(&["-p", "--port"], argparse::Store, "Specify port");
        ap.refer(&mut host).add_option(&["-c", "--connect"], argparse::StoreOption, "Start client, connect to specified host");
        ap.refer(&mut nickname).add_option(&["--nick"], argparse::StoreOption, "Nickname");
        ap.refer(&mut console).add_option(&["--console"], argparse::StoreTrue, "Console version");
        ap.refer(&mut start_server).add_option(&["-s", "--server"], argparse::StoreTrue, "Start server");
        ap.parse_args_or_exit();
    }

    #[cfg(target_os = "emscripten")]
    {
        host = Some(String::from("play.kuviman.com"));
        nickname = Some(String::from("nickname"));
    }
    #[cfg(not(target_os = "emscripten"))]
    {
        if start_server {
            if host.is_some() {
                std::thread::spawn(move || { server::run(port) });
            } else {
                server::run(port);
            }
        } else if host.is_none() {
            host = Some(String::from("play.kuviman.com"));
        }
    }
    if let Some(host) = host {
        if start_server {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        if console {
            #[cfg(not(target_os = "emscripten"))]
            console_client::run(&host, port);
        } else {
            *HOST.lock().unwrap() = host;
            *PORT.lock().unwrap() = port;
            {
                let mut nick = NICK.lock().unwrap();
                if let Some(nickname) = nickname {
                    *nick = nickname;
                } else {
                    print!("nick: ");
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    std::io::stdin().read_line(&mut nick).unwrap();
                }
                *nick = nick.trim().to_owned();
            }
            codevisual::run::<client::TrollInvasion>();
        }
    }
}
