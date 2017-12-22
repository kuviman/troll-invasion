use ::*;

pub struct GameLobby {
    app: Rc<codevisual::App>,
    nick: String,
    menu: MenuScreen,
    sender: connection::Sender,
    ready: bool,
    players: BTreeMap<String, bool>,
    play_type: PlayType,
}

const READY_INDEX: usize = 5;
const PLAYERS_START: usize = 8;

impl Screen for GameLobby {
    fn handle(&mut self, event: Event) -> Option<Box<Screen>> {
        match event {
            Event::Draw(framebuffer) => {
                self.menu.sections.split_off(PLAYERS_START);
                let back_color = Color::rgb(0.1, 0.1, 0.1);
                for (player, &ready) in &self.players {
                    self.menu.sections.push(MenuSection {
                        text: format!("{}: {}", player, if ready { "ready" } else { "not ready" }),
                        size: 7.0,
                        color: if ready { Color::rgb(0.5, 1.0, 0.5) } else { Color::rgb(1.0, 0.5, 0.5) },
                        back_color,
                        hover_color: None,
                    });
                }
                while self.menu.sections.len() < PLAYERS_START + 4 {
                    let mut section = MenuSection::new_empty(7.0, back_color);
                    if self.menu.sections.len() == PLAYERS_START {
                        section.text = String::from("nobody connected yet");
                    }
                    self.menu.sections.push(section);
                }

                self.menu.draw(framebuffer);
            }
            Event::Message(message) => {
                match message {
                    ServerMessage::ReadyStatus { nick, ready } => {
                        if nick == self.nick {
                            self.ready = ready;
                        } else {
                            self.players.insert(nick, ready);
                        }
                    }
                    ServerMessage::GameStart => {
                        return Some(Box::new(Game::new(&self.app, self.nick.clone(), self.sender.clone())));
                    }
                    ServerMessage::GameLeft { nick } => {
                        if nick == self.nick {
                            return Some(Box::new(Lobby::new(&self.app, self.nick.clone(), self.sender.clone())));
                        } else {
                            self.players.remove(&nick);
                        }
                    }
                    _ => {}
                }
            }
            Event::Event(event) => {
                if let Some(selection) = self.menu.handle(event) {
                    if selection == 2 {
                        self.sender.send("leaveGame");
                    } else if selection == READY_INDEX {
                        self.ready = !self.ready;
                        self.menu.sections[READY_INDEX] = ready_section(self.ready, self.play_type);
                        self.sender.send(if self.ready { "ready" } else { "unready" });
                    }
                }
            }
            _ => {}
        }
        None
    }
}

fn ready_section(ready: bool, play_type: PlayType) -> MenuSection {
    match play_type {
        PlayType::Player => MenuSection {
            text: if ready { "ready".to_owned() } else { "not ready".to_owned() },
            size: 10.0,
            color: Color::WHITE,
            back_color: if ready { Color::rgb(0.0, 0.5, 0.0) } else { Color::rgb(0.5, 0.0, 0.0) },
            hover_color: Some(Color::rgb(0.5, 0.5, 0.5)),
        },
        PlayType::Spectator => MenuSection {
            text: String::from("spectator"),
            size: 10.0,
            color: Color::WHITE,
            back_color: Color::BLACK,
            hover_color: None,
        }
    }
}

impl GameLobby {
    pub fn new(app: &Rc<codevisual::App>, nick: String, game_name: String, sender: connection::Sender, typ: PlayType) -> Self {
        Self {
            app: app.clone(),
            nick: nick.clone(),
            sender,
            ready: false,
            players: BTreeMap::new(),
            play_type: typ,
            menu: MenuScreen::new(app, vec![
                MenuSection {
                    text: String::from("TroLL InvaSioN"),
                    size: 20.0,
                    color: Color::rgb(0.8, 0.8, 1.0),
                    back_color: Color::BLACK,
                    hover_color: None,
                },
                MenuSection {
                    text: nick.clone(),
                    size: 5.0,
                    color: Color::rgb(0.7, 0.7, 0.7),
                    back_color: Color::BLACK,
                    hover_color: None,
                },
                MenuSection {
                    text: game_name.clone(),
                    size: 5.0,
                    color: Color::WHITE,
                    back_color: Color::BLACK,
                    hover_color: Some(Color::RED),
                },
                MenuSection::new_empty(1.0, Color::rgb(0.05, 0.05, 0.05)),
                MenuSection::new_empty(10.0, Color::BLACK),
                ready_section(false, typ),
                MenuSection::new_empty(10.0, Color::BLACK),
                MenuSection {
                    text: String::from("players:"),
                    size: 5.0,
                    color: Color::rgb(0.5, 0.5, 0.5),
                    back_color: Color::BLACK,
                    hover_color: None,
                }]),
        }
    }
}