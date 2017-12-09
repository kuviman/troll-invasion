use ::*;

pub struct NicknameScreen {
    app: Rc<codevisual::App>,
    menu: MenuScreen,
}

impl NicknameScreen {
    pub fn new(app: &Rc<codevisual::App>) -> Self {
        Self {
            app: app.clone(),
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
                    text: String::from("nickname:"),
                    size: 5.0,
                    color: Color::WHITE,
                    back_color: Color::BLACK,
                    hover_color: None,
                },
                MenuSection {
                    text: String::new(),
                    size: 10.0,
                    color: Color::WHITE,
                    back_color: Color::rgb(0.2, 0.2, 0.4),
                    hover_color: None,
                },
                MenuSection {
                    text: String::from("play!"),
                    size: 10.0,
                    color: Color::WHITE,
                    back_color: Color::rgb(0.3, 0.3, 0.3),
                    hover_color: Some(Color::rgb(0.5, 0.5, 1.0)),
                }]),
        }
    }
    fn nick_section(&mut self) -> &mut MenuSection {
        &mut self.menu.sections[4]
    }
}

impl Screen for NicknameScreen {
    fn handle(&mut self, event: Event) -> Option<Box<Screen>> {
        match event {
            Event::Event(event) => {
                if let codevisual::Event::KeyDown { key } = event {
                    match key {
                        codevisual::Key::Backspace => {
                            self.nick_section().text.pop();
                        }
                        codevisual::Key::Enter => {
                            if !self.nick_section().text.is_empty() {
                                *NICK.lock().unwrap() = self.nick_section().text.clone();
                                return Some(connect(&self.app));
                            }
                        }
                        _ => {
                            let key = format!("{:?}", key);
                            if key.len() == 1 {
                                let nick_section = self.nick_section();
                                if nick_section.text.len() < 15 {
                                    nick_section.text += &key.to_lowercase();
                                }
                            }
                        }
                    }
                } else if let Some(selection) = self.menu.handle(event) {
                    if self.menu.sections[selection].text == "play!" {
                        if !self.nick_section().text.is_empty() {
                            *NICK.lock().unwrap() = self.nick_section().text.clone();
                            return Some(connect(&self.app));
                        }
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