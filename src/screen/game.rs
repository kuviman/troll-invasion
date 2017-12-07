use ::*;

#[derive(Vertex)]
struct Vertex {
    a_pos: Vec2<f32>,
}

pub struct Game {
    nick: String,
    hex_geometry: ugli::VertexBuffer<Vertex>,
    map: Vec<Vec<Option<GameCell>>>,
    material: codevisual::Material,
    app: Rc<codevisual::Application>,
    current_player: String,
    energy_left: Option<usize>,
    selected_cell: Option<Vec2<usize>>,
    hovered_cell: Option<Vec2<usize>>,
    matrix: std::cell::Cell<Mat4<f32>>,
    ui: Ui,
    ready_button: conrod::widget::Id,
    skip_button: conrod::widget::Id,
    current_status: conrod::widget::Id,
    sender: connection::Sender,
}

impl Screen for Game {
    fn handle(&mut self, event: Event) -> Option<Box<Screen>> {
        match event {
            Event::Event(event) => self.handle_event(event),
            Event::Draw(framebuffer) => self.draw(framebuffer),
            Event::Update(delta_time) => self.update(delta_time),
            Event::Message(message) => self.handle_message(message),
        }
        None
    }
}

impl Game {
    pub fn new(app: &Rc<codevisual::Application>, nick: String, sender: connection::Sender) -> Self {
        let mut ui = Ui::new(app);
        let skip_button = ui.widget_id_generator().next();
        let ready_button = ui.widget_id_generator().next();
        let current_status = ui.widget_id_generator().next();
        Self {
            hovered_cell: None,
            app: app.clone(),
            nick,
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
            ui,
            skip_button,
            ready_button,
            current_status,
            energy_left: None,
            sender,
        }
    }

    fn handle_message(&mut self, message: ServerMessage) {
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
                self.energy_left = None;
            }
            EnergyLeft(energy) => {
                self.energy_left = Some(energy);
            }
            _ => {}
        }
    }

    fn update(&mut self, delta_time: f64) {}

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer,
                    Some(if self.nick == self.current_player {
                        Color::rgb(0.0, 0.1, 0.0)
                    } else {
                        Color::rgb(0.1, 0.0, 0.0)
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
                        if Some(vec2(i, j)) == self.hovered_cell {
                            self.hex(framebuffer,
                                     center,
                                     2.0 / 3.0,
                                     Color::rgb(1.0, 1.0, 1.0));
                        }
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

        let current_status = if self.map.is_empty() {
            String::from("Getting ready")
        } else {
            format!("{}'s turn: {}", self.current_player, match self.energy_left {
                None => String::from("Attack phase"),
                Some(energy) => format!("Upgrade phase ({} energy left)", energy),
            })
        };

        use conrod::{Widget, Positionable, Sizeable, Labelable, Colorable};
        let mut news = Vec::new();
        {
            let ui = &mut self.ui.set_widgets();
            for _ in conrod::widget::Button::new()
                .mid_left_with_margin_on(ui.window, 50.0)
                .w_h(150.0, 50.0)
                .label("READY")
                .set(self.ready_button, ui) {
                news.push("ready");
            }
            for _ in conrod::widget::Button::new()
                .down_from(self.ready_button, 10.0)
                .w_h(150.0, 50.0)
                .label("Next phase")
                .set(self.skip_button, ui) {
                news.push("next phase");
            }
            conrod::widget::Text::new(&current_status)
                .mid_bottom_with_margin_on(ui.window, 50.0)
                .color(conrod::color::WHITE)
                .set(self.current_status, ui);
        }
        self.ui.draw(framebuffer);
        for new in news {
            self.sender.send(new);
        }
    }

    fn handle_event(&mut self, event: codevisual::Event) {
        let window_size = self.app.window().get_size();
        self.ui.handle_event(event.clone());
        match event {
            codevisual::Event::MouseDown { button: codevisual::MouseButton::Left, position: pos } => {
                if !self.map.is_empty() {
                    if let Some(Vec2 { x, y }) = self.find_pos(vec2(pos.x as f32, pos.y as f32)) {
                        self.sender.send(format!("{} {}", x, y));
                    }
                }
            }
            codevisual::Event::MouseMove { position: pos } => {
                self.hovered_cell = self.find_pos(vec2(pos.x as f32, pos.y as f32));
            }
            _ => {}
        }
    }
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
                       depth_func: None,
                       blend_mode: Some(ugli::BlendMode::Alpha),
                       ..Default::default()
                   });
    }
}