use ::*;

pub struct WinnerScreen {
    app: Rc<codevisual::App>,
    nick: String,
    menu: MenuScreen,
    sender: connection::Sender,
}

impl WinnerScreen {
    pub fn new(app: &Rc<codevisual::App>, nick: String, winner: String, sender: connection::Sender) -> Self {
        Self {
            app: app.clone(),
            nick,
            sender,
            menu: MenuScreen::new(app, vec![
                MenuSection {
                    text: String::from("TroLL InvaSioN"),
                    size: 20.0,
                    color: Color::rgb(0.8, 0.8, 1.0),
                    back_color: Color::BLACK,
                    hover_color: None,
                },
                MenuSection::new_empty(1.0, Color::rgb(0.05, 0.05, 0.05)),
                MenuSection::new_empty(10.0, Color::BLACK),
                MenuSection {
                    text: String::from("winner:"),
                    size: 5.0,
                    color: Color::WHITE,
                    back_color: Color::BLACK,
                    hover_color: None,
                },
                MenuSection {
                    text: winner,
                    size: 10.0,
                    color: Color::WHITE,
                    back_color: Color::BLACK,
                    hover_color: None,
                },
                MenuSection {
                    text: String::from("leave"),
                    size: 10.0,
                    color: Color::WHITE,
                    back_color: Color::rgb(0.3, 0.3, 0.3),
                    hover_color: Some(Color::rgb(0.5, 0.5, 1.0)),
                }]),
        }
    }
}

impl Screen for WinnerScreen {
    fn handle(&mut self, event: Event) -> Option<Box<Screen>> {
        match event {
            Event::Event(event) => {
                if let codevisual::Event::KeyDown { key: codevisual::Key::Enter } = event {
                    return Some(Box::new(Lobby::new(&self.app, self.nick.clone(), self.sender.clone())));
                } else if let Some(selection) = self.menu.handle(event) {
                    if self.menu.sections[selection].text == "leave" {
                        self.sender.send("leaveGame");
                        return Some(Box::new(Lobby::new(&self.app, self.nick.clone(), self.sender.clone())));
                    }
                }
            }
            Event::Draw(framebuffer) => {
                self.menu.draw(framebuffer);
            }
            _ => {}
        }
        None
    }
}