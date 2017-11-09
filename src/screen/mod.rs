use ::*;

mod game;
mod lobby;

pub use self::game::*;
pub use self::lobby::*;

pub enum Event<'a, 'b> where 'b: 'a {
    Update(f64),
    Draw(&'a mut ugli::Framebuffer<'b>),
    Event(codevisual::Event),
    Message(ServerMessage),
}

pub trait Screen {
    fn handle(&mut self, event: Event) -> Option<Box<Screen>>;
}