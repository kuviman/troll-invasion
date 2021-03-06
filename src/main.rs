#[cfg(not(target_os = "emscripten"))]
extern crate ws;
#[cfg(not(target_os = "emscripten"))]
extern crate argparse;
#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate codevisual;
#[macro_use]
extern crate ugli;
#[cfg(target_os = "emscripten")]
#[macro_use]
extern crate webby;

pub(crate) use codevisual::prelude::*;

#[cfg(not(target_os = "emscripten"))]
mod server;
mod screen;
mod model;
mod connection;

pub(crate) use model::*;
pub(crate) use screen::*;

lazy_static! {
    static ref HOST: Mutex<String> = Mutex::new(String::new());
    static ref PORT: Mutex<u16> = Mutex::new(0);
    static ref NICK: Mutex<String> = Mutex::new(String::new());
    static ref RECEIVER: Mutex<Option<connection::Receiver>> = Mutex::new(None);
}

static mut RESOURCES: *const Resources = 0 as _;

fn resources() -> &'static Resources {
    unsafe { &*RESOURCES }
}

struct TrollInvasion {
    screen: Box<screen::Screen>,
}

#[derive(Resources)]
struct Resources {
    #[path = "data/troll.png"]
    troll_texture: ugli::Texture2d,
    #[path = "data/ground.jpg"]
    ground_texture: ugli::Texture2d,
    #[path = "data/grass.jpg"]
    grass_texture: ugli::Texture2d,
    #[path = "data/tree.png"]
    tree_texture: ugli::Texture2d,
}

impl codevisual::Game for TrollInvasion {
    type Resources = Resources;
    fn get_title() -> String {
        String::from("Troll invasion")
    }
    fn update(&mut self, delta_time: f64) {
        if let Some(screen) = self.screen.handle(screen::Event::Update(delta_time)) {
            self.screen = screen;
        }
        let receiver = RECEIVER.lock().unwrap();
        if let Some(ref receiver) = *receiver {
            while let Some(message) = receiver.try_recv() {
                if let Some(screen) = self.screen.handle(screen::Event::Message(message)) {
                    self.screen = screen;
                }
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
    fn new(app: &Rc<codevisual::App>, mut resources: Resources) -> Self {
        resources.ground_texture.set_wrap_mode(ugli::WrapMode::Repeat);
        resources.grass_texture.set_wrap_mode(ugli::WrapMode::Repeat);
        unsafe { RESOURCES = Box::into_raw(Box::new(resources)); }
        Self {
            screen: Box::new(NicknameScreen::new(app)),
        }
    }
}

fn connect(app: &Rc<codevisual::App>) -> Box<Screen> {
    let (sender, receiver) = connection::connect(&NICK.lock().unwrap(), &HOST.lock().unwrap(), *PORT.lock().unwrap());
    *RECEIVER.lock().unwrap() = Some(receiver);
    Box::new(screen::Lobby::new(app, NICK.lock().unwrap().clone(), sender))
}

const DEFAULT_PORT: u16 = 8008;

#[cfg(target_os = "emscripten")]
fn main() {
    webby::emscripten::run_script(include_str!("web.js"));
    *PORT.lock().unwrap() = DEFAULT_PORT;
    *HOST.lock().unwrap() = String::from("play.kuviman.com");
    codevisual::run::<TrollInvasion>();
}

#[cfg(not(target_os = "emscripten"))]
fn main() {
    std::env::set_current_dir("static").unwrap();
    let mut port: u16 = DEFAULT_PORT;
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

    if start_server {
        if host.is_some() {
            std::thread::spawn(move || { server::run(port) });
        } else {
            server::run(port);
        }
    } else if host.is_none() {
        host = Some(String::from("play.kuviman.com"));
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
            }
            *nick = nick.trim().to_owned();
        }
        codevisual::run::<TrollInvasion>();
    }
}
