use ::*;

#[derive(Vertex)]
struct Vertex {
    a_pos: Vec2<f32>,
}

pub struct Game {
    nick: String,
    font: codevisual::Font,
    hex_geometry: ugli::VertexBuffer<Vertex>,
    player_colors: HashMap<String, char>,
    player_hovers: HashMap<String, Vec2<usize>>,
    map: Vec<Vec<Option<GameCell>>>,
    material: codevisual::Material,
    app: Rc<codevisual::App>,
    current_player: String,
    energy_left: Option<usize>,
    selected_cell: Option<Vec2<usize>>,
    hovered_cell: Option<Vec2<usize>>,
    matrix: std::cell::Cell<Mat4<f32>>,
    can_moves: Vec<Vec2<usize>>,
    sender: connection::Sender,
}

impl Screen for Game {
    fn handle(&mut self, event: Event) -> Option<Box<Screen>> {
        match event {
            Event::Event(event) => self.handle_event(event),
            Event::Draw(framebuffer) => self.draw(framebuffer),
            Event::Update(delta_time) => self.update(delta_time),
            Event::Message(message) => { return self.handle_message(message); }
        }
        None
    }
}

pub fn player_color(color: char) -> Color {
    match color {
        'A' => Color::RED,
        'B' => Color::GREEN,
        'C' => Color::BLUE,
        'D' => Color::YELLOW,
        'E' => Color::MAGENTA,
        'F' => Color::CYAN,
        _ => unreachable!("Do not have that much colors")
    }
}

const LEAVE_SIZE: f32 = 2.0;
const LEAVE_OFFSET: f32 = 2.0;

const STATUS_SIZE: f32 = 4.0;
const STATUS_OFFSET: f32 = 2.0;

impl Game {
    pub fn new(app: &Rc<codevisual::App>, nick: String, sender: connection::Sender) -> Self {
        Self {
            can_moves: Vec::new(),
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
            energy_left: None,
            sender,
            font: codevisual::Font::new(app.ugli_context(), (include_bytes!("font.ttf") as &[u8]).to_owned()),
            player_colors: HashMap::new(),
            player_hovers: HashMap::new(),
        }
    }

    fn handle_message(&mut self, message: ServerMessage) -> Option<Box<Screen>> {
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
            DeselectCell => {
                self.selected_cell = None;
            }
            Turn { nick } => {
                self.current_player = nick;
                self.energy_left = None;
            }
            EnergyLeft(energy) => {
                self.energy_left = Some(energy);
            }
            GameFinish { winner } => {
                return Some(Box::new(WinnerScreen::new(
                    &self.app, self.nick.clone(),
                    winner.clone(),
                    player_color(self.player_colors[&winner]),
                    self.sender.clone())));
            }
            GameLeft { nick } => {
                if nick == self.nick {
                    return Some(Box::new(Lobby::new(&self.app, self.nick.clone(), self.sender.clone())));
                }
            }
            PlayerColor { nick, color } => {
                self.player_colors.insert(nick, color);
            }
            HoverCell { nick, row, col } => {
                if nick != self.nick {
                    self.player_hovers.insert(nick, vec2(row, col));
                }
            }
            HoverNone { nick } => {
                self.player_hovers.remove(&nick);
            }
            CanMove { cells } => {
                self.can_moves = cells;
            }
            _ => {}
        }
        None
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
                        if self.can_moves.contains(&vec2(i, j)) {
                            self.hex(framebuffer, center, 2.0 / 3.0,
                                     Color::rgba(0.5, 0.5, 0.5, 0.5));
                        }
                        for (name, &cell) in &self.player_hovers {
                            if vec2(i, j) == cell {
                                self.hex(framebuffer,
                                         center,
                                         2.0 / 3.0,
                                         self.player_colors.get(name).map_or(
                                             Color::rgba(1.0, 1.0, 1.0, 0.1),
                                             |&c| Color {
                                                 alpha: 0.5,
                                                 ..player_color(c)
                                             }));
                            }
                        }
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
                            for index in 0..count {
                                let pos = center + Vec2::rotated(vec2(0.3, 0.0), (index as f32 / count as f32) * 2.0 * std::f32::consts::PI);
                                let size = 0.05;
                                self.hex(framebuffer, pos, size, player_color(owner));
                            }
                        }
                    }
                }
            }
        }

        let framebuffer_size = framebuffer.get_size();

        let unit = framebuffer_size.y as f32 / 100.0;
        self.font.draw_aligned(
            framebuffer,
            "leave",
            vec2(framebuffer_size.x as f32 - LEAVE_OFFSET * unit,
                 framebuffer_size.y as f32 - (LEAVE_OFFSET + LEAVE_SIZE) * unit),
            1.0, LEAVE_SIZE * unit,
            if self.leave_rect_hover() {
                Color::RED
            } else {
                Color::WHITE
            });

        if !self.current_player.is_empty() {
            let current_status = format!("{}'s turn: {}", self.current_player, match self.energy_left {
                None => String::from("Attack phase"),
                Some(energy) => format!("Upgrade phase ({} energy left)", energy),
            });
            if self.status_hover() {
                self.font.draw_aligned(
                    framebuffer,
                    if self.energy_left.is_none() { "next phase" } else { "end turn" },
                    vec2(framebuffer_size.x as f32 / 2.0, STATUS_OFFSET * unit),
                    0.5, STATUS_SIZE * unit, Color::WHITE);
            } else {
                self.font.draw_aligned(
                    framebuffer,
                    &current_status,
                    vec2(framebuffer_size.x as f32 / 2.0, STATUS_OFFSET * unit),
                    0.5, STATUS_SIZE * unit, player_color(self.player_colors[&self.current_player]));
            }
        }
    }

    fn status_hover(&self) -> bool {
        let window_size = self.app.window().get_size();
        let cursor_pos = self.app.window().get_cursor_position();
        cursor_pos.y as f32 > window_size.y as f32 * (1.0 - (STATUS_SIZE * 2.0 + STATUS_OFFSET) / 100.0)
    }

    fn leave_rect_hover(&self) -> bool {
        let window_size = self.app.window().get_size();
        let cursor_pos = self.app.window().get_cursor_position();
        let cursor_pos = vec2(cursor_pos.x as f32, window_size.y as f32 - cursor_pos.y as f32);
        let unit = window_size.y as f32 / 100.0;
        let rect = Rect::from_corners(
            vec2(window_size.x as f32 - LEAVE_OFFSET * 2.0 * unit - self.font.measure("leave", LEAVE_SIZE * unit).unwrap().width(),
                 window_size.y as f32 - LEAVE_OFFSET * 2.0 * unit - LEAVE_SIZE * unit),
            vec2(window_size.x as f32, window_size.y as f32));
        rect.contains(cursor_pos)
    }

    fn handle_event(&mut self, event: codevisual::Event) {
        let window_size = self.app.window().get_size();
        match event {
            codevisual::Event::MouseDown { button: codevisual::MouseButton::Left, position: pos } => {
                if self.leave_rect_hover() {
                    self.sender.send("leaveGame");
                } else if self.status_hover() {
                    self.sender.send("next phase");
                } else if !self.map.is_empty() {
                    if let Some(Vec2 { x, y }) = self.find_pos(vec2(pos.x as f32, pos.y as f32)) {
                        self.sender.send(format!("{} {}", x, y));
                    }
                }
            }
            codevisual::Event::MouseDown { button: codevisual::MouseButton::Right, position: pos } => {
                if let Some(Vec2 { x, y }) = self.find_pos(vec2(pos.x as f32, pos.y as f32)) {
                    self.sender.send(format!("fullUp {} {}", x, y));
                }
            }
            codevisual::Event::MouseMove { position: pos } => {
                let cell = self.find_pos(vec2(pos.x as f32, pos.y as f32));
                if self.hovered_cell != cell {
                    self.sender.send(match cell {
                        None => format!("hover none"),
                        Some(pos) => format!("hover {} {}", pos.x, pos.y),
                    });
                    self.hovered_cell = cell;
                }
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