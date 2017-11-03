extern crate ws;
extern crate env_logger;
extern crate argparse;
#[macro_use]
extern crate lazy_static;
extern crate regex;

#[macro_use]
extern crate codevisual;

pub ( crate ) use codevisual::prelude::*;
pub ( crate ) use codevisual::ugli;

fn server(port: u16) {
    let mut jar = std::process::Command::new("java");
    jar.arg("-jar").arg("TrollInvasion.jar");
    jar.stdin(std::process::Stdio::piped());
    jar.stdout(std::process::Stdio::piped());
    let jar = jar.spawn().expect("Failed to start TrollInvasion.jar");
    let jar_in = std::rc::Rc::new(std::cell::RefCell::new(jar.stdin.unwrap()));
    let mut jar_out = std::io::BufReader::new(jar.stdout.unwrap());
    let connections: std::sync::Arc<std::sync::Mutex<Vec<ws::Sender>>> = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    std::thread::spawn({
        let connections = connections.clone();
        move || {
            let mut line = String::new();
            loop {
                use std::io::BufRead;
                line.clear();
                if jar_out.read_line(&mut line).unwrap() == 0 {
                    break;
                }
                let line = line.trim_right_matches('\n').trim_right_matches('\r');
                eprintln!("> {}", line);
                for connection in connections.lock().unwrap().iter() {
                    connection.send(line).unwrap();
                }
            }
        }
    });
    ws::listen(("0.0.0.0", port), move |connection| {
        connections.lock().unwrap().push(connection.clone());
        let jar_in = jar_in.clone();
        move |message: ws::Message| {
            let message = message.into_text().unwrap();
            eprintln!("< {}", message);
            use std::io::Write;
            let mut jar_in = jar_in.borrow_mut();
            jar_in.write(message.as_bytes()).unwrap();
            jar_in.write("\n".as_bytes()).unwrap();
            jar_in.flush().unwrap();
            Ok(())
        }
    }).unwrap();
}

fn console_client(server: &str, port: u16) {
    ws::connect(format!("ws://{}:{}", server, port), |connection| {
        struct Handler { nick: String, connection: ws::Sender }
        impl ws::Handler for Handler {
            fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
                print!("nick: ");
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                let mut nick = String::new();
                std::io::stdin().read_line(&mut nick).unwrap();
                let nick = nick.trim().to_owned();
                self.nick = nick.clone();

                let connection = self.connection.clone();
                connection.send(format!("+{}", nick)).unwrap();

                std::thread::spawn(move || {
                    loop {
                        let mut line = String::new();
                        std::io::stdin().read_line(&mut line).unwrap();
                        connection.send(format!("{}:{}", nick, line.trim())).unwrap();
                    }
                });

                Ok(())
            }
            fn on_message(&mut self, message: ws::Message) -> ws::Result<()> {
                println!("{}", message.into_text().unwrap());
                Ok(())
            }
        }
        Handler { nick: String::from("<unnamed>"), connection }
    }).unwrap();
}

enum ServerMessage {
    ReadyStatus {
        nick: String,
        ready: bool,
    },
    MapLine(usize, Vec<Option<Cell>>),
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
                        "##" => Some(Cell::Empty),
                        "__" => None,
                        _ => {
                            let (count, owner) = cell.split_at(cell.len() - 1);
                            let count = count.parse().unwrap();
                            let owner = owner.parse().unwrap();
                            Some(Cell::Populated {
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
enum Cell {
    Empty,
    Populated {
        count: usize,
        owner: char,
    }
}

struct TrollInvasionClient {
    nick: String,
    connection: Arc<Mutex<Option<ws::Sender>>>,
    receiver: std::sync::mpsc::Receiver<ServerMessage>,
    map: Vec<Vec<Option<Cell>>>,
    material: codevisual::Material,
    app: Rc<codevisual::Application>,
    current_player: String,
    selected_cell: Option<Vec2<usize>>,
}

impl codevisual::Game for TrollInvasionClient {
    type Resources = ();

    fn new(app: &Rc<codevisual::Application>, resources: Self::Resources) -> Self {
        let nick = NICK.lock().unwrap().clone();
        let connection = Arc::new(Mutex::new(None));
        let (sender, receiver) = std::sync::mpsc::channel();
        thread::spawn({
            let connection = connection.clone();
            let nick = nick.clone();
            move || {
                let address = format!("ws://{}:{}", *HOST.lock().unwrap(), *PORT.lock().unwrap());
                println!("Connecting to {}", address);
                ws::connect(address, |conn| {
                    struct Handler {
                        nick: String,
                        sender: std::sync::mpsc::Sender<ServerMessage>,
                        connection: Arc<Mutex<Option<ws::Sender>>>,
                        conn: ws::Sender,
                    }
                    impl ws::Handler for Handler {
                        fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
                            self.conn.send(format!("+{}", self.nick)).unwrap();
                            *self.connection.lock().unwrap() = Some(self.conn.clone());
                            Ok(())
                        }
                        fn on_message(&mut self, message: ws::Message) -> ws::Result<()> {
                            let message = message.into_text().unwrap();
                            println!("{}", message);
                            let message = ServerMessage::parse(&message);
                            self.sender.send(message).unwrap();
                            Ok(())
                        }
                    }
                    Handler {
                        nick: nick.clone(),
                        sender: sender.clone(),
                        connection: connection.clone(),
                        conn,
                    }
                }).unwrap();
            }
        });
        Self {
            app: app.clone(),
            nick,
            connection,
            receiver,
            current_player: String::new(),
            map: Vec::new(),
            selected_cell: None,
            material: codevisual::Material::new(app.ugli_context(), (), (), include_str!("shader.glsl")),
        }
    }

    fn update(&mut self, delta_time: f64) {
        while let Ok(message) = self.receiver.try_recv() {
            use ServerMessage::*;
            match message {
                MapLine(index, line) => {
                    while index >= self.map.len() {
                        self.map.push(Vec::new());
                    }
                    self.map[index] = line;
                }
                UpgradePhase => {
                    self.selected_cell = None;
                }
                SelectCell { row, col } => {
                    self.selected_cell = Some(vec2(row, col));
                }
                Turn { nick } => {
                    self.current_player = nick;
                }
                _ => {}
            }
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer,
                    Some(if self.nick == self.current_player {
                        Color::rgb(0.0, 0.3, 0.0)
                    } else {
                        Color::BLACK
                    }), None);
        if !self.map.is_empty() {
            let width = self.map[0].len();
            let height = self.map.len();

            let conv = |pos: Vec2<f32>| {
                vec2((pos.x / height as f32) * 2.0 - 1.0, 1.0 - (pos.y / width as f32) * 2.0)
            };

            for (i, line) in self.map.iter().enumerate() {
                for (j, cell) in line.iter().enumerate() {
                    const OFF: Vec2<f32> = Vec2 { x: 0.1, y: 0.1 };
                    if let Some(cell) = *cell {
                        let p1 = conv(vec2(j as f32, i as f32) + OFF);
                        let p2 = conv(vec2((j + 1) as f32, (i + 1) as f32) - OFF);
                        let radius = min((p1.x - p2.x).abs(), (p1.y - p2.y).abs()) / 2.5;
                        let center = (p1 + p2) / 2.0;
                        self.quad(framebuffer, p1, p2,
                                  if self.selected_cell.map_or(false, |pos| pos == vec2(i, j)) {
                                      Color::rgb(0.5, 0.5, 0.5)
                                  } else {
                                      Color::rgb(0.2, 0.2, 0.2)
                                  });
                        if let Cell::Populated { count, owner } = cell {
                            let color = match owner {
                                'A' => Color::rgb(1.0, 0.0, 0.0),
                                'B' => Color::rgb(0.0, 1.0, 0.0),
                                'C' => Color::rgb(0.0, 0.0, 1.0),
                                'D' => Color::rgb(1.0, 1.0, 0.0),
                                'E' => Color::rgb(0.0, 1.0, 1.0),
                                'F' => Color::rgb(1.0, 0.0, 1.0),
                                _ => unreachable!()
                            };
                            for index in 0..count {
                                let pos = center + Vec2::rotated(vec2(radius, 0.0), (index as f32 / count as f32) * 2.0 * std::f32::consts::PI);
                                let size = vec2(radius / 10.0, radius / 10.0);
                                self.quad(framebuffer, pos + size, pos - size, color);
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_event(&mut self, event: codevisual::Event) {
        match event {
            codevisual::Event::KeyDown { key } => {
                match key {
                    codevisual::Key::Space => {
                        self.send("ready");
                    }
                    codevisual::Key::S => {
                        self.send("next phase");
                    }
                    _ => {}
                }
            }
            codevisual::Event::MouseDown { button: codevisual::MouseButton::Left, position: mut pos } => {
                pos.x /= self.app.window().get_size().x as f64;
                pos.y /= self.app.window().get_size().y as f64;
                if !self.map.is_empty() {
                    pos.x *= self.map[0].len() as f64;
                    pos.y *= self.map.len() as f64;
                    let row = clamp(pos.y as usize, 0, self.map.len());
                    let col = clamp(pos.x as usize, 0, self.map[0].len());
                    self.send(format!("{} {}", row, col));
                }
            }
            _ => {}
        }
    }
}

impl TrollInvasionClient {
    fn quad(&self, framebuffer: &mut ugli::Framebuffer, p1: Vec2<f32>, p2: Vec2<f32>, color: Color) {
        ugli::draw(framebuffer,
                   &self.material.ugli_program(),
                   ugli::Quad::DRAW_MODE,
                   &**ugli::quad(self.app.ugli_context()),
                   uniforms!(p1: p1, p2: p2, u_color: color),
                   ugli::DrawParameters {
                       depth_test: ugli::DepthTest::Off,
                       blend_mode: ugli::BlendMode::Off,
                       ..Default::default()
                   });
    }
    fn send<S: std::borrow::Borrow<str>>(&mut self, message: S) {
        if let Some(connection) = self.connection.lock().unwrap().as_ref() {
            connection.send(format!("{}:{}", self.nick, message.borrow())).unwrap();
        }
    }
}

lazy_static! {
    static ref HOST: Mutex<String> = Mutex::new(String::new());
    static ref PORT: Mutex<u16> = Mutex::new(0);
    static ref NICK: Mutex<String> = Mutex::new(String::new());
}

fn main() {
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

    if start_server {
        if host.is_some() {
            std::thread::spawn(move || { server(port) });
        } else {
            server(port);
        }
    } else if host.is_none() {
        host = Some(String::from("play.kuviman.com"));
    }
    if let Some(host) = host {
        if start_server {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        if console {
            console_client(&host, port);
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
            codevisual::run::<TrollInvasionClient>();
        }
    }
}
