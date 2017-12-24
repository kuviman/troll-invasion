use ::*;

pub struct Lobby {
    app: Rc<codevisual::App>,
    nick: String,
    menu: MenuScreen,
    sender: connection::Sender,
    next_query_time: f64,
    games: BTreeMap<String, usize>,
}

const CREATE_INDEX: usize = 6;
const GAMES_START: usize = 9;

impl Screen for Lobby {
    fn handle(&mut self, event: Event) -> Option<Box<Screen>> {
        match event {
            Event::Update(delta_time) => {
                self.next_query_time -= delta_time;
                if self.next_query_time < 0.0 {
                    self.sender.send("listGames");
                    self.next_query_time = 1.0;
                }
            }
            Event::Draw(framebuffer) => {
                self.menu.sections.split_off(GAMES_START);
                for (game, player_count) in &self.games {
                    self.menu.sections.push(MenuSection {
                        text: format!("{} ({})", game, player_count),
                        size: 7.0,
                        color: Color::WHITE,
                        back_color: Color::rgb(0.3, 0.3, 0.3),
                        hover_color: Some(Color::rgb(0.5, 0.5, 1.0)),
                    });
                }
                if self.games.is_empty() {
                    self.menu.sections.push(MenuSection {
                        text: String::from("no games yet, create one!"),
                        size: 7.0,
                        color: Color::rgb(0.5, 0.5, 0.5),
                        back_color: Color::BLACK,
                        hover_color: None,
                    });
                }
                self.menu.draw(framebuffer);
            }
            Event::Message(message) => {
                match message {
                    ServerMessage::GameList { name, player_count } => {
                        if self.games.contains_key(&name) {
                            *self.games.get_mut(&name).unwrap() = player_count;
                        } else {
                            self.games.insert(name, player_count);
                        }
                    }
                    ServerMessage::GameEntered { name, typ } => {
                        return Some(Box::new(GameLobby::new(&self.app, self.nick.clone(), name, self.sender.clone(), typ)));
                    }
                    _ => {}
                }
            }
            Event::Event(event) => {
                if let codevisual::Event::KeyDown { key } = event {
                    match key {
                        codevisual::Key::Backspace => {
                            self.name_section().text.pop();
                        }
                        codevisual::Key::Enter => {
                            if !self.name_section().text.is_empty() {
                                self.create_game();
                            }
                        }
                        _ => {
                            let key = format!("{:?}", key);
                            if key.len() == 1 {
                                let name_section = self.name_section();
                                if name_section.text.len() < 15 {
                                    name_section.text += &key.to_lowercase();
                                }
                            }
                        }
                    }
                } else if let Some(selection) = self.menu.handle(event.clone()) {
                    if selection == 1 {
                        self.sender.send("-");
                        *RECEIVER.lock().unwrap() = None;
                        return Some(Box::new(NicknameScreen::new(&self.app)));
                    } else if selection == CREATE_INDEX {
                        if !self.name_section().text.is_empty() {
                            self.create_game();
                        }
                    } else if selection >= GAMES_START && !self.games.is_empty() {
                        self.connect(selection - GAMES_START);
                    }
                } else if let codevisual::Event::MouseDown { button: codevisual::MouseButton::Right, position } = event {
                    if let Some(selection) = self.menu.handle(codevisual::Event::MouseDown {
                        button: codevisual::MouseButton::Left,
                        position,
                    }) {
                        if selection >= GAMES_START && !self.games.is_empty() {
                            self.connect_spectator(selection - GAMES_START);
                        }
                    }
                }
            }
        }
        None
    }
}

impl Lobby {
    fn create_game(&mut self) {
        let name = self.name_section().text.clone();
        self.sender.send(format!("createGame {}", name));
    }
    fn connect(&mut self, index: usize) {
        let game_name = self.games.keys().nth(index).unwrap();
        self.sender.send(format!("joinGame {} player", game_name));
    }
    fn connect_spectator(&mut self, index: usize) {
        let game_name = self.games.keys().nth(index).unwrap();
        self.sender.send(format!("joinGame {} spectator", game_name));
    }
    fn name_section(&mut self) -> &mut MenuSection {
        &mut self.menu.sections[CREATE_INDEX - 1]
    }
    pub fn new(app: &Rc<codevisual::App>, nick: String, sender: connection::Sender) -> Self {
        Self {
            app: app.clone(),
            nick: nick.clone(),
            next_query_time: 2.0,
            sender,
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
                    color: Color::WHITE,
                    back_color: Color::BLACK,
                    hover_color: Some(Color::RED),
                },
                MenuSection::new_empty(1.0, Color::rgb(0.05, 0.05, 0.05)),
                MenuSection::new_empty(10.0, Color::BLACK),
                MenuSection {
                    text: String::from("game name:"),
                    size: 5.0,
                    color: Color::WHITE,
                    back_color: Color::BLACK,
                    hover_color: None,
                },
                MenuSection {
                    text: String::from("newgame"),
                    size: 10.0,
                    color: Color::WHITE,
                    back_color: Color::rgb(0.2, 0.2, 0.4),
                    hover_color: None,
                },
                MenuSection {
                    text: String::from("create game"),
                    size: 10.0,
                    color: Color::WHITE,
                    back_color: Color::rgb(0.3, 0.3, 0.3),
                    hover_color: Some(Color::rgb(0.5, 0.5, 1.0)),
                },
                MenuSection::new_empty(10.0, Color::BLACK),
                MenuSection {
                    text: String::from("coNnecT"),
                    size: 10.0,
                    color: Color::WHITE,
                    back_color: Color::BLACK,
                    hover_color: None,
                }]),
            games: BTreeMap::new(),
        }
    }
}