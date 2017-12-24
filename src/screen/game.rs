use ::*;

#[derive(Vertex)]
struct Vertex {
    a_pos: Vec2<f32>,
}

pub struct Game {
    nick: String,
    font: codevisual::Font,
    hex_geometry: ugli::VertexBuffer<Vertex>,
    quad_geometry: ugli::VertexBuffer<Vertex>,
    player_colors: HashMap<String, char>,
    player_hovers: HashMap<String, Vec2<usize>>,
    map: Vec<Vec<Option<GameCell>>>,
    next_frame_time: f64,
    next_map: Vec<Vec<Option<GameCell>>>,
    map_queue: std::collections::VecDeque<Vec<Vec<Option<GameCell>>>>,
    material: codevisual::Material,
    app: Rc<codevisual::App>,
    current_player: String,
    matrix: Cell<Mat4<f32>>,
    energy_left: Option<usize>,
    selected_cell: Option<Vec2<usize>>,
    hovered_cell: Option<Vec2<usize>>,
    camera_pos: Vec2<f32>,
    camera_dist: f32,
    can_moves: Vec<Vec2<usize>>,
    sender: connection::Sender,
    troll_material: codevisual::Material,
    dragging: bool,
    start_drag: Option<Vec2>,
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
            dragging: false,
            start_drag: None,
            can_moves: Vec::new(),
            hovered_cell: None,
            next_frame_time: 0.0,
            app: app.clone(),
            nick,
            matrix: Cell::new(Mat4::identity()),
            current_player: String::new(),
            map: Vec::new(),
            next_map: Vec::new(),
            selected_cell: None,
            material: codevisual::Material::new(app.ugli_context(), (), (), include_str!("shader.glsl")),
            troll_material: codevisual::Material::new(app.ugli_context(), (), (), include_str!("troll.glsl")),
            hex_geometry: ugli::VertexBuffer::new_static(app.ugli_context(), {
                let mut vs = Vec::new();
                for i in 0..6 {
                    vs.push(Vertex {
                        a_pos: Vec2::rotated(vec2(0.0, 1.0), i as f32 / 6.0 * 2.0 * std::f32::consts::PI)
                    });
                }
                vs
            }),
            quad_geometry: ugli::VertexBuffer::new_static(app.ugli_context(), vec![
                Vertex { a_pos: vec2(-1.0, 0.0) },
                Vertex { a_pos: vec2(1.0, 0.0) },
                Vertex { a_pos: vec2(1.0, 2.0) },
                Vertex { a_pos: vec2(-1.0, 2.0) }, ]),
            energy_left: None,
            sender,
            font: codevisual::Font::new(app.ugli_context(), (include_bytes!("font.ttf") as &[u8]).to_owned()),
            player_colors: HashMap::new(),
            player_hovers: HashMap::new(),
            map_queue: std::collections::VecDeque::new(),
            camera_pos: vec2(0.0, 0.0),
            camera_dist: 1.5,
        }
    }

    fn handle_message(&mut self, message: ServerMessage) -> Option<Box<Screen>> {
        use ServerMessage::*;
        match message {
            MapLine(index, line) => {
                self.next_map.push(line);
            }
            EndMap => {
                self.map_queue.push_back(mem::replace(&mut self.next_map, Vec::new()));
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

    fn update(&mut self, delta_time: f64) {
        self.next_frame_time -= delta_time;
        if self.next_frame_time < 0.0 {
            if let Some(map) = self.map_queue.pop_front() {
                self.map = map;
                self.next_frame_time = 0.1;
            }
        }
        let mut dv: Vec2<f32> = vec2(0.0, 0.0);
        if self.app.window().is_key_pressed(codevisual::Key::W) {
            dv.y += 1.0;
        }
        if self.app.window().is_key_pressed(codevisual::Key::A) {
            dv.x -= 1.0;
        }
        if self.app.window().is_key_pressed(codevisual::Key::S) {
            dv.y -= 1.0;
        }
        if self.app.window().is_key_pressed(codevisual::Key::D) {
            dv.x += 1.0;
        }
        self.camera_pos += dv * delta_time as f32;
    }

    fn projection_matrix(&self) -> Mat4<f32> {
        let aspect = self.app.window().get_size().x as f32 / self.app.window().get_size().y as f32;
        Mat4::perspective(std::f32::consts::PI / 2.0, aspect, 0.1, 100.0)
    }

    fn view_matrix(&self) -> Mat4<f32> {
        let (width, height) = (self.map[0].len() as f32 / 3.0.sqrt(), self.map.len() as f32);
        Mat4::translate(vec3(0.0, 0.0, -self.camera_dist)) *
            Mat4::rotate_x(-0.2) *
            Mat4::translate(-self.camera_pos.extend(0.0)) *
            Mat4::scale_uniform(2.0 / max(width, height)) *
            Mat4::translate(vec3(-width / 2.0, -height / 2.0, 0.0))
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer,
                    Some(if self.nick == self.current_player {
                        Color::rgb(0.0, 0.1, 0.0)
                    } else {
                        Color::rgb(0.1, 0.0, 0.0)
                    }), Some(1.0));
        if !self.map.is_empty() {
            self.matrix.set(self.projection_matrix() * self.view_matrix());
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
                    }
                }
            }
            for (i, line) in self.map.iter().enumerate() {
                for (j, cell) in line.iter().enumerate() {
                    if let Some(cell) = *cell {
                        let center = vec2((j as f32 + 0.5) / 3.0.sqrt(), i as f32 + 0.5);
                        if let GameCell::Populated { count, owner } = cell {
                            for index in 0..count {
                                let pos = center + Vec2::rotated(vec2(0.3, 0.0), (index as f32 / count as f32) * 2.0 * std::f32::consts::PI);
                                let size = 0.05;
                                self.draw_troll(framebuffer, pos, player_color(owner));
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
            codevisual::Event::MouseUp { button: codevisual::MouseButton::Left, position: pos } => {
                self.start_drag = None;
                if self.dragging {
                    self.dragging = false;
                } else if self.leave_rect_hover() {
                    self.sender.send("leaveGame");
                } else if self.status_hover() {
                    self.sender.send("next phase");
                } else if !self.map.is_empty() {
                    if let Some(Vec2 { x, y }) = self.find_pos(vec2(pos.x as f32, pos.y as f32)) {
                        self.sender.send(format!("{} {}", x, y));
                    }
                }
            }
            codevisual::Event::MouseDown { button: codevisual::MouseButton::Left, position: pos } => {
                self.start_drag = Some(pos);
            }
            codevisual::Event::MouseDown { button: codevisual::MouseButton::Right, position: pos } => {
                if let Some(Vec2 { x, y }) = self.find_pos(vec2(pos.x as f32, pos.y as f32)) {
                    self.sender.send(format!("fullUp {} {}", x, y));
                }
            }
            codevisual::Event::MouseMove { position: pos } => {
                let mut captured = false;
                if let Some(start) = self.start_drag {
                    if self.dragging {
                        let dv = (pos - start) / self.app.window().get_size().y as f64 * 2.0 * self.camera_dist as f64;
                        self.camera_pos.x -= dv.x as f32;
                        self.camera_pos.y += dv.y as f32;
                        self.start_drag = Some(pos);
                        captured = true;
                    } else if (start - pos).len() > 10.0 {
                        self.dragging = true;
                        captured = true;
                    }
                }
                if !captured {
                    let cell = self.find_pos(vec2(pos.x as f32, pos.y as f32));
                    if self.hovered_cell != cell {
                        self.sender.send(match cell {
                            None => format!("hover none"),
                            Some(pos) => format!("hover {} {}", pos.x, pos.y),
                        });
                        self.hovered_cell = cell;
                    }
                }
            }
            codevisual::Event::Wheel { delta } => {
                self.camera_dist = clamp(self.camera_dist * (1.0 - delta as f32 / 1000.0), 0.3, 3.0);
            }
            _ => {}
        }
    }
    fn find_pos(&self, pos: Vec2<f32>) -> Option<Vec2<usize>> {
        let pos = vec2((pos.x * 2.0 / self.app.window().get_size().x as f32 - 1.0),
                       (1.0 - pos.y * 2.0 / self.app.window().get_size().y as f32));
        let matrix = self.matrix.get();
        for (i, line) in self.map.iter().enumerate() {
            for (j, cell) in line.iter().enumerate() {
                if let Some(cell) = *cell {
                    let center = vec2((j as f32 + 0.5) / 3.0.sqrt(), i as f32 + 0.5);
                    let mut inside = true;
                    for i in 0..6 {
                        let p1 = self.hex_geometry[i].a_pos * 2.0 / 3.0 + center;
                        let p2 = self.hex_geometry[(i + 1) % 6].a_pos * 2.0 / 3.0 + center;
                        let p1 = matrix * p1.extend(0.0).extend(1.0);
                        let p1 = vec2(p1.x / p1.w, p1.y / p1.w);
                        let p2 = matrix * p2.extend(0.0).extend(1.0);
                        let p2 = vec2(p2.x / p2.w, p2.y / p2.w);
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
    fn draw_troll(&self, framebuffer: &mut ugli::Framebuffer, pos: Vec2<f32>, color: Color) {
        let proj = self.projection_matrix();
        let view = self.view_matrix();
        ugli::draw(framebuffer,
                   &self.troll_material.ugli_program(),
                   ugli::DrawMode::TriangleFan,
                   &self.quad_geometry,
                   uniforms!(u_pos: pos, u_color: color, u_projection_matrix: proj, u_view_matrix: view, u_texture: unsafe { &*TROLL_TEXTURE }),
                   ugli::DrawParameters {
                       depth_func: Some(default()),
                       blend_mode: Some(ugli::BlendMode::Alpha),
                       ..Default::default()
                   });
    }
}