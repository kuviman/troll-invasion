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
mod ui;
mod screen;
mod model;
mod connection;

pub ( crate ) use model::*;
pub ( crate ) use screen::*;
pub ( crate ) use ui::Ui;

lazy_static! {
    static ref HOST: Mutex<String> = Mutex::new(String::new());
    static ref PORT: Mutex<u16> = Mutex::new(0);
    static ref NICK: Mutex<String> = Mutex::new(String::new());
}

struct TrollInvasion {
    screen: Box<screen::Screen>,
    receiver: connection::Receiver,
}

impl codevisual::Game for TrollInvasion {
    type Resources = ();
    fn get_title() -> String {
        String::from("Troll invasion")
    }
    fn update(&mut self, delta_time: f64) {
        if let Some(screen) = self.screen.handle(screen::Event::Update(delta_time)) {
            self.screen = screen;
        }
        while let Some(message) = self.receiver.try_recv() {
            if let Some(screen) = self.screen.handle(screen::Event::Message(message)) {
                self.screen = screen;
            }
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let event = screen::Event::Draw(framebuffer);
        if let Some(screen) = self.screen.handle(event) {
            self.screen = screen;
        }
    }
    fn handle_event(&mut self, event: codevisual::Event) {
        if let Some(screen) = self.screen.handle(screen::Event::Event(event)) {
            self.screen = screen;
        }
    }
    fn new(app: &Rc<codevisual::Application>, resources: Self::Resources) -> Self {
        let (sender, receiver) = connection::connect(&NICK.lock().unwrap(), &HOST.lock().unwrap(), *PORT.lock().unwrap());
        Self {
            receiver,
            screen: Box::new(screen::Lobby::new(app, NICK.lock().unwrap().clone(), sender)),
        }
    }
}

fn main() {
    #[cfg(target_os = "emscripten")]
    web::run_script(troll_invasion_web::JS_SOURCE);

    #[cfg(not(target_os = "emscripten"))]
    env_logger::init().unwrap();

    let mut port: u16 = 8008;
    let mut host = None;
    let mut start_server = false;
    let mut nickname = None;

    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("TrollInvasion client/server. By default starts client connecting to play.kuviman.com.");
        ap.refer(&mut port).add_option(&["-p", "--port"], argparse::Store, "Specify port");
        ap.refer(&mut host).add_option(&["-c", "--connect"], argparse::StoreOption, "Start client, connect to specified host");
        ap.refer(&mut nickname).add_option(&["--nick"], argparse::StoreOption, "Nickname");
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
        codevisual::run::<TrollInvasion>();
    }
}
