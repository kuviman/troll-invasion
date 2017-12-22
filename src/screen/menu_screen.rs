use ::*;

pub struct MenuSection {
    pub text: String,
    pub size: f32,
    pub color: Color,
    pub back_color: Color,
    pub hover_color: Option<Color>,
}

impl MenuSection {
    pub fn new_empty(size: f32, color: Color) -> Self {
        Self {
            text: String::new(),
            size,
            color: Color::WHITE,
            back_color: color,
            hover_color: None,
        }
    }
}

#[derive(Vertex)]
struct Vertex {
    a_pos: Vec2<f32>,
}

pub struct MenuScreen {
    app: Rc<codevisual::App>,
    font: codevisual::Font,
    color_material: codevisual::Material,
    geometry: RefCell<ugli::VertexBuffer<Vertex>>,
    pub back_color: Color,
    pub sections: Vec<MenuSection>,
    hover_index: Option<usize>,
}

impl MenuScreen {
    pub fn new(app: &Rc<codevisual::App>, sections: Vec<MenuSection>) -> Self {
        Self {
            app: app.clone(),
            font: codevisual::Font::new(app.ugli_context(), (include_bytes!("font.ttf") as &[u8]).to_owned()),
            color_material: codevisual::Material::new(app.ugli_context(), (), (), include_str!("color.glsl")),
            geometry: RefCell::new(ugli::VertexBuffer::new_dynamic(app.ugli_context(), Vec::new())),
            back_color: Color::BLACK,
            sections,
            hover_index: None,
        }
    }
    pub fn update(&mut self, delta_time: f64) {}
    pub fn handle(&mut self, event: codevisual::Event) -> Option<usize> {
        match event {
            codevisual::Event::MouseMove { position } => {
                self.hover_index = self.hover(position);
            }
            codevisual::Event::MouseDown { position, button: codevisual::MouseButton::Left } => {
                return self.hover(position);
            }
            _ => {}
        }
        None
    }
    fn hover(&self, position: Vec2) -> Option<usize> {
        let position = 100.0 - 100.0 * position.y as f32 / self.app.window().get_size().y as f32;
        let sum_size: f32 = self.sections.iter().map(|s| s.size).sum();
        let mut pos = 50.0 + sum_size / 2.0;
        for (index, section) in self.sections.iter().enumerate() {
            if pos - section.size < position && position < pos {
                return Some(index);
            }
            pos -= section.size;
        }
        None
    }
    pub fn draw(&self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(self.back_color), None);
        let sum_size: f32 = self.sections.iter().map(|s| s.size).sum();
        let mut pos = 50.0 + sum_size / 2.0;
        for (index, section) in self.sections.iter().enumerate() {
            self.draw_section(index, section, framebuffer, pos);
            pos -= section.size;
        }
    }
    pub fn draw_rect(&self, framebuffer: &mut ugli::Framebuffer, p1: Vec2<f32>, p2: Vec2<f32>, color: Color) {
        {
            let mut geometry = self.geometry.borrow_mut();
            geometry.clear();
            geometry.push(Vertex { a_pos: p1 });
            geometry.push(Vertex { a_pos: vec2(p1.x, p2.y) });
            geometry.push(Vertex { a_pos: p2 });
            geometry.push(Vertex { a_pos: vec2(p2.x, p1.y) });
        }
        ugli::draw(framebuffer,
                   &self.color_material.ugli_program(),
                   ugli::DrawMode::TriangleFan,
                   &*self.geometry.borrow(),
                   uniforms!(u_color: color),
                   ugli::DrawParameters {
                       depth_func: None,
                       ..default()
                   });
    }
    fn draw_section(&self, index: usize, section: &MenuSection, framebuffer: &mut ugli::Framebuffer, pos: f32) {
        let pos = pos / 100.0;
        let frame_size = framebuffer.get_size();
        let frame_size = vec2(frame_size.x as f32, frame_size.y as f32);
        let y1 = pos * 2.0 - 1.0;
        let y2 = y1 - section.size / 100.0 * 2.0;
        self.draw_rect(framebuffer, vec2(-1.0, y1), vec2(1.0, y2), section.back_color);
        self.font.draw_aligned(framebuffer,
                               &section.text,
                               vec2(frame_size.x / 2.0, frame_size.y * (pos - section.size / 100.0)),
                               0.5,
                               frame_size.y * section.size / 100.0 * 0.6,
                               {
                                   let mut color = section.color;
                                   if let Some(hover_color) = section.hover_color {
                                       if let Some(hover_index) = self.hover_index {
                                           if hover_index == index {
                                               color = hover_color;
                                           }
                                       }
                                   }
                                   color
                               });
    }
}