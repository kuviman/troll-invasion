use ::*;

#[derive(Vertex)]
struct Vertex {
    a_pos: Vec2<f32>,
}

pub struct TrollInvasion {
    nick: String,
    hex_geometry: ugli::VertexBuffer<Vertex>,
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
            hex_geometry: ugli::VertexBuffer::new_static(app.ugli_context(), {
                let mut vs = Vec::new();
                for i in 0..6 {
                    vs.push(Vertex {
                        a_pos: Vec2::rotated(vec2(0.0, 1.0), i as f32 / 6.0 * 2.0 * std::f32::consts::PI)
                    });
                }
                vs
            }),
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
            let (width, height) = (self.map[0].len() as f32 / 3.0.sqrt(), self.map.len() as f32);
            let aspect = self.app.window().get_size().x as f32 / self.app.window().get_size().y as f32;
            self.matrix.set(Mat4::scale_uniform(2.0 / max(width, height)) *
                Mat4::scale(vec3(1.0 / aspect, -1.0, 1.0) * 0.8) *
                Mat4::translate(vec3(-width / 2.0, -height / 2.0, 0.0)));
            for (i, line) in self.map.iter().enumerate() {
                for (j, cell) in line.iter().enumerate() {
                    if let Some(cell) = *cell {
                        let center = vec2((j as f32 + 0.5) / 3.0.sqrt(), i as f32 + 0.5);
                        self.hex(framebuffer,
                                 center,
                                 2.0 / 3.0 - 0.05,
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
                                self.hex(framebuffer, pos, size, color);
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
            codevisual::Event::KeyDown { key: codevisual::Key::S } => {
                self.send("next phase");
            }
            codevisual::Event::MouseDown { button: codevisual::MouseButton::Left, position: pos } => {
                if !self.map.is_empty() {
                    if let Some(Vec2 { x, y }) = self.find_pos(vec2(pos.x as f32, pos.y as f32)) {
                        self.send(format!("{} {}", x, y));
                    }
                }
            }
            _ => {}
        }
    }
}

impl TrollInvasion {
    fn find_pos(&self, pos: Vec2<f32>) -> Option<Vec2<usize>> {
        let pos = vec2((pos.x * 2.0 / self.app.window().get_size().x as f32 - 1.0),
                       (1.0 - pos.y * 2.0 / self.app.window().get_size().y as f32));
        let pos = self.matrix.get().inverse() * pos.extend(0.0).extend(1.0);
        let pos = vec2(pos.x, pos.y);
        for (i, line) in self.map.iter().enumerate() {
            for (j, cell) in line.iter().enumerate() {
                if let Some(cell) = *cell {
                    let center = vec2((j as f32 + 0.5) / 3.0.sqrt(), i as f32 + 0.5);
                    let mut inside = true;
                    for i in 0..6 {
                        let p1 = self.hex_geometry[i].a_pos * 2.0 / 3.0 + center;
                        let p2 = self.hex_geometry[(i + 1) % 6].a_pos * 2.0 / 3.0 + center;
                        if Vec2::cross(p2 - p1, pos - p1) < 0.0 {
                            inside = false;
                            break;
                        }
                    }
                    if inside {
                        return Some(vec2(i, j));
                    }
                }
            }
        }
        None
    }
    fn hex(&self, framebuffer: &mut ugli::Framebuffer, pos: Vec2<f32>, radius: f32, color: Color) {
        ugli::draw(framebuffer,
                   &self.material.ugli_program(),
                   ugli::DrawMode::TriangleFan,
                   &self.hex_geometry,
                   uniforms!(u_radius: radius, u_pos: pos, u_color: color, u_matrix: self.matrix.get()),
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