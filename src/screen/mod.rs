use ::*;

mod menu_screen;
mod game;
mod lobby;
mod nickname;

pub use self::menu_screen::*;
pub use self::game::*;
pub use self::lobby::*;
pub use self::nickname::*;

pub enum Event<'a, 'b> where 'b: 'a {
    Update(f64),
    Draw(&'a mut ugli::Framebuffer<'b>),
    Event(codevisual::Event),
    Message(ServerMessage),
}

pub trait Screen {
    fn handle(&mut self, event: Event) -> Option<Box<Screen>>;
}