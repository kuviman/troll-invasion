use ::*;

#[derive(Vertex, Debug)]
struct Vertex {
    a_pos: Vec2<f32>,
    a_vt: Vec2<f32>,
    a_color: Color,
}

pub struct UiRenderer {
    app: Rc<codevisual::Application>,
    material: codevisual::Material,
    geometry: ugli::VertexBuffer<Vertex>,
    cache: conrod::text::GlyphCache,
    cache_texture: ugli::Texture2d,
}

impl UiRenderer {
    pub fn new(app: &Rc<codevisual::Application>) -> Self {
        Self {
            app: app.clone(),
            material: codevisual::Material::new(app.ugli_context(), (), (), include_str!("shader.glsl")),
            geometry: ugli::VertexBuffer::new_dynamic(app.ugli_context(), Vec::new()),
            cache: conrod::text::GlyphCache::new(2048, 2048, 0.1, 0.1),
            cache_texture: ugli::Texture2d::new_uninitialized(app.ugli_context(), vec2(2048, 2048)),
        }
    }
    pub fn render(&mut self, framebuffer: &mut ugli::Framebuffer, mut primitives: conrod::render::Primitives) {
        {
            use conrod::render::PrimitiveWalker;
            let geometry: &mut Vec<Vertex> = &mut self.geometry;
            geometry.clear();
            while let Some(primitive) = primitives.next_primitive() {
                let conrod::render::Primitive { kind, rect, .. } = primitive;
                match kind {
                    conrod::render::PrimitiveKind::Rectangle { color } => {
                        let p1 = rect.top_left();
                        let p2 = rect.bottom_right();
                        let x1 = p1[0] as f32;
                        let x2 = p2[0] as f32;
                        let y1 = p1[1] as f32;
                        let y2 = p2[1] as f32;
                        let conrod::color::Rgba(r, g, b, a) = color.to_rgb();
                        let color = Color::rgba(r, g, b, a);
                        geometry.push(Vertex {
                            a_pos: vec2(x1, y1),
                            a_vt: vec2(-1.0, -1.0),
                            a_color: color,
                        });
                        geometry.push(Vertex {
                            a_pos: vec2(x2, y1),
                            a_vt: vec2(-1.0, -1.0),
                            a_color: color,
                        });
                        geometry.push(Vertex {
                            a_pos: vec2(x2, y2),
                            a_vt: vec2(-1.0, -1.0),
                            a_color: color,
                        });

                        geometry.push(Vertex {
                            a_pos: vec2(x1, y1),
                            a_vt: vec2(-1.0, -1.0),
                            a_color: color,
                        });
                        geometry.push(Vertex {
                            a_pos: vec2(x2, y2),
                            a_vt: vec2(-1.0, -1.0),
                            a_color: color,
                        });
                        geometry.push(Vertex {
                            a_pos: vec2(x1, y2),
                            a_vt: vec2(-1.0, -1.0),
                            a_color: color,
                        });
                    }
                    conrod::render::PrimitiveKind::TrianglesSingleColor { triangles, color } => {
                        let conrod::color::Rgba(r, g, b, a) = color;
                        let color = Color::rgba(r, g, b, a);
                        for triangle in triangles {
                            let mut add_v = |pos: [f64; 2]| {
                                geometry.push(Vertex {
                                    a_pos: vec2(pos[0] as f32, pos[1] as f32),
                                    a_vt: vec2(-1.0, -1.0),
                                    a_color: color,
                                });
                            };
                            add_v(triangle.0[0]);
                            add_v(triangle.0[1]);
                            add_v(triangle.0[2]);
                        }
                    }
                    conrod::render::PrimitiveKind::Image { .. } => println!("Image"),
                    conrod::render::PrimitiveKind::Text { color, text, font_id } => {
                        let conrod::color::Rgba(r, g, b, a) = color.to_rgb();
                        let color = Color::rgba(r, g, b, a);
                        let glyphs = text.positioned_glyphs(1.0);
                        for glyph in glyphs {
                            self.cache.queue_glyph(font_id.index(), glyph.clone());
                        }
                        let texture = &mut self.cache_texture;
                        self.cache.cache_queued(|rect, data| {
                            let x = rect.min.x as usize;
                            let y = rect.min.y as usize;
                            let width = rect.width() as usize;
                            let height = rect.height() as usize;

                            let mut fixed_data = Vec::with_capacity(data.len() * 4);
                            for byte in data {
                                for _ in 0..4 {
                                    fixed_data.push(*byte);
                                }
                            }

                            unsafe {
                                texture.sub_image(vec2(x, y), vec2(width, height), &fixed_data);
                            }
                        }).unwrap();
                        for glyph in glyphs {
                            if let Some((texture_rect, rect)) = self.cache.rect_for(font_id.index(), glyph).unwrap() {
                                let x1 = rect.min.x as f32;
                                let y1 = rect.min.y as f32;
                                let x2 = rect.max.x as f32;
                                let y2 = rect.max.y as f32;
//                                println!("{} {}", x1, y1);
                                let u1 = texture_rect.min.x;
                                let u2 = texture_rect.max.x;
                                let v1 = texture_rect.min.y;
                                let v2 = texture_rect.max.y;
                                geometry.push(Vertex {
                                    a_pos: vec2(x1, y1),
                                    a_vt: vec2(u1, v1),
                                    a_color: color,
                                });
                                geometry.push(Vertex {
                                    a_pos: vec2(x2, y1),
                                    a_vt: vec2(u2, v1),
                                    a_color: color,
                                });
                                geometry.push(Vertex {
                                    a_pos: vec2(x2, y2),
                                    a_vt: vec2(u2, v2),
                                    a_color: color,
                                });

                                geometry.push(Vertex {
                                    a_pos: vec2(x1, y1),
                                    a_vt: vec2(u1, v1),
                                    a_color: color,
                                });
                                geometry.push(Vertex {
                                    a_pos: vec2(x2, y2),
                                    a_vt: vec2(u2, v2),
                                    a_color: color,
                                });
                                geometry.push(Vertex {
                                    a_pos: vec2(x1, y2),
                                    a_vt: vec2(u1, v2),
                                    a_color: color,
                                });
                            }
                        }
                    }
                    conrod::render::PrimitiveKind::TrianglesMultiColor { .. } => println!("TrianglesMultiColor"),
                    conrod::render::PrimitiveKind::Other(..) => {}
                }
            }
        }
        ugli::draw(framebuffer,
                   &self.material.ugli_program(),
                   //                   ugli::DrawMode::LineStrip { line_width: 1.0 },
                   ugli::DrawMode::Triangles,
                   &self.geometry,
                   uniforms!(u_glyph_cache: &self.cache_texture),
                   ugli::DrawParameters {
                       depth_test: ugli::DepthTest::Off,
                       blend_mode: ugli::BlendMode::Alpha,
                       ..Default::default()
                   });
    }
}