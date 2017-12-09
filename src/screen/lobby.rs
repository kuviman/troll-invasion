use ::*;

pub struct Lobby {
    app: Rc<codevisual::App>,
    nick: String,
    ui: Ui,
    sender: connection::Sender,
    game_name: String,
    game_name_widget: conrod::widget::Id,
    create_game_button: conrod::widget::Id,
    game_list_widget: conrod::widget::Id,
    next_query_time: f64,
    games: HashMap<String, usize>,
}

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
                ugli::clear(framebuffer, Some(Color::BLACK), None);
                {
                    use conrod::{Widget, Positionable, Sizeable, Labelable, Colorable};
                    let ui = &mut self.ui.set_widgets();
                    for event in conrod::widget::TextBox::new(&self.game_name)
                        .middle_of(ui.window)
                        .w_h(150.0, 50.0)
                        .set(self.game_name_widget, ui) {
                        match event {
                            conrod::widget::text_box::Event::Update(s) => self.game_name = s,
                            _ => {}
                        }
                    }
                    for _ in conrod::widget::Button::new()
                        .label("Create game")
                        .down_from(self.game_name_widget, 10.0)
                        .set(self.create_game_button, ui) {
                        self.sender.send(format!("createGame {}", self.game_name));
                        return Some(Box::new(screen::Game::new(&self.app, self.nick.clone(), self.sender.clone())));
                    }
                    let (mut events, _) = conrod::widget::ListSelect::single(self.games.len())
                        .down_from(self.create_game_button, 10.0)
                        .set(self.game_list_widget, ui);
                    while let Some(event) = events.next(ui, |_| false) {
                        let mut games = self.games.iter().collect::<Vec<_>>();
                        match event {
                            conrod::widget::list_select::Event::Item(conrod::widget::list::Item { i, widget_id, .. }) => {
                                let game = games[i];
                                conrod::widget::Text::new(&format!("{} ({} players)", game.0, game.1))
                                    .color(conrod::color::WHITE)
                                    .set(widget_id, ui);
                            }
                            conrod::widget::list_select::Event::Selection(selection) => {
                                self.sender.send(format!("joinGame {}", games[selection].0));
                                return Some(Box::new(screen::Game::new(&self.app, self.nick.clone(), self.sender.clone())));
                            }
                            _ => {}
                        }
                    }
                }
                self.ui.draw(framebuffer);
            }
            Event::Message(message) => {
                if let ServerMessage::GameList { name, player_count } = message {
                    if self.games.contains_key(&name) {
                        *self.games.get_mut(&name).unwrap() = player_count;
                    } else {
                        self.games.insert(name, player_count);
                    }
                }
            }
            Event::Event(event) => {
                self.ui.handle_event(event);
            }
        }
        None
    }
}

impl Lobby {
    pub fn new(app: &Rc<codevisual::App>, nick: String, sender: connection::Sender) -> Self {
        let mut ui = Ui::new(app);
        let game_name_widget = ui.widget_id_generator().next();
        let create_game_button = ui.widget_id_generator().next();
        let game_list_widget = ui.widget_id_generator().next();
        Self {
            app: app.clone(),
            nick,
            game_name: String::new(),
            next_query_time: 0.0,
            game_name_widget,
            create_game_button,
            game_list_widget,
            sender,
            ui,
            games: HashMap::new(),
        }
    }
}