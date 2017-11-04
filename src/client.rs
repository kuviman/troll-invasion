use ::*;

pub struct TrollInvasion {
    nick: String,
    connection: Arc<Mutex<Option<ws::Sender>>>,
    receiver: std::sync::mpsc::Receiver<ServerMessage>,
    map: Vec<Vec<Option<GameCell>>>,
    material: codevisual::Material,
    app: Rc<codevisual::Application>,
    current_player: String,
    selected_cell: Option<Vec2<usize>>,
    matrix: std::cell::Cell<Mat4<f32>>,
}

impl codevisual::Game for TrollInvasion {
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
            matrix: std::cell::Cell::new(Mat4::identity()),
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
            let (width, height) = (self.map[0].len(), self.map.len());
            self.matrix.set(Mat4::scale(vec3(1.0, -1.0, 1.0)) *
                Mat4::translate(vec3(-1.0, -1.0, 0.0)) *
                Mat4::scale(vec3(2.0 / width as f32, 2.0 / height as f32, 1.0)));
            for (i, line) in self.map.iter().enumerate() {
                for (j, cell) in line.iter().enumerate() {
                    const OFF: Vec2<f32> = Vec2 { x: 0.1, y: 0.1 };
                    if let Some(cell) = *cell {
                        let center = vec2(j as f32 + 0.5, i as f32 + 0.5);
                        self.quad(framebuffer,
                                  vec2(j as f32, i as f32) + OFF,
                                  vec2((j + 1) as f32, (i + 1) as f32) - OFF,
                                  if self.selected_cell.map_or(false, |pos| pos == vec2(i, j)) {
                                      Color::rgb(0.5, 0.5, 0.5)
                                  } else {
                                      Color::rgb(0.2, 0.2, 0.2)
                                  });
                        if let GameCell::Populated { count, owner } = cell {
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
                                let pos = center + Vec2::rotated(vec2(0.3, 0.0), (index as f32 / count as f32) * 2.0 * std::f32::consts::PI);
                                let size = 0.05;
                                let size = vec2(size, size);
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
            codevisual::Event::KeyDown { key: codevisual::Key::Space } => {
                self.send("ready");
            }
            codevisual::Event::MouseDown { button: codevisual::MouseButton::Left, position: pos } => {
                let pos = vec2((pos.x * 2.0 / self.app.window().get_size().x as f64 - 1.0) as f32,
                               (1.0 - pos.y * 2.0 / self.app.window().get_size().y as f64) as f32);
                let pos = self.matrix.get().inverse() * pos.extend(0.0).extend(1.0);
                if !self.map.is_empty() {
                    let row = clamp(pos.y as usize, 0, self.map.len());
                    let col = clamp(pos.x as usize, 0, self.map[0].len());
                    self.send(format!("{} {}", row, col));
                }
            }
            _ => {}
        }
    }
}

impl TrollInvasion {
    fn quad(&self, framebuffer: &mut ugli::Framebuffer, p1: Vec2<f32>, p2: Vec2<f32>, color: Color) {
        ugli::draw(framebuffer,
                   &self.material.ugli_program(),
                   ugli::Quad::DRAW_MODE,
                   &**ugli::quad(self.app.ugli_context()),
                   uniforms!(p1: p1, p2: p2, u_color: color, u_matrix: self.matrix.get()),
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