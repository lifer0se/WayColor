use eframe::glow;

use crate::color::Color;

#[derive(Debug, Clone, PartialEq)]
pub enum GradientType {
    Gradient,
    Slider(String),
}

#[derive(Debug)]
pub struct Gradient {
    gtype: GradientType,
    program: glow::Program,
    vertex_array: glow::VertexArray,
}

impl Gradient {
    pub fn new(gl: &glow::Context, gtype: GradientType) -> Self {
        use glow::HasContext as _;

        unsafe {
            let program = gl.create_program().expect("Cannot create program");
            let (vertex_shader_source, fragment_shader_source) = get_shader_sources(&gtype);
            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];

            let shaders: Vec<_> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(shader, shader_source);
                    gl.compile_shader(shader);
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Failed to compile {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );
                    gl.attach_shader(program, shader);
                    shader
                })
                .collect();

            gl.link_program(program);
            assert!(
                gl.get_program_link_status(program),
                "{}",
                gl.get_program_info_log(program)
            );

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");

            Self {
                gtype,
                program,
                vertex_array,
            }
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }

    pub fn paint(&self, gl: &glow::Context, color: Color) {
        use glow::HasContext as _;
        unsafe {
            gl.use_program(Some(self.program));
            match &self.gtype {
                GradientType::Gradient => gl.uniform_4_f32(
                    gl.get_uniform_location(self.program, "hue").as_ref(),
                    color.float_by_name("r"),
                    color.float_by_name("g"),
                    color.float_by_name("b"),
                    1.0,
                ),
                GradientType::Slider(_) => gl.uniform_4_f32(
                    gl.get_uniform_location(self.program, "color").as_ref(),
                    color.float_by_name("r"),
                    color.float_by_name("g"),
                    color.float_by_name("b"),
                    1.0,
                ),
            }
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
    }
}

fn get_shader_sources(gtype: &GradientType) -> (String, String) {
    let shader_version = if cfg!(target_arch = "wasm32") {
        "#version 300 es"
    } else {
        "#version 330"
    };

    let vertex_shader_source = r#"
            const vec2 verts[6] = vec2[6](
                vec2(-1.0, 1.0),
                vec2(1.0, 1.0),
                vec2(1.0, -1.0),
                vec2(1.0, -1.0),
                vec2(-1.0, 1.0),
                vec2(-1.0, -1.0)
            );
            out vec2 tex_coord;
            void main() {
                gl_Position = vec4(verts[gl_VertexID], 0.0, 1.0);
                tex_coord = gl_Position.xy * 0.5 + 0.5;
            }
        "#;

    let hsv2rgb = r#"
            vec4 hsv2rgb(float h, float s, float v, float a) {
                float c = v * s;
                float x = c * (1.0 - abs(mod(h * 6.0, 2.0) - 1.0));
                float m = v - c;

                vec3 rgb;

                if (h < 1.0/6.0) {
                    rgb = vec3(c, x, 0.0);
                } else if (h < 2.0/6.0) {
                    rgb = vec3(x, c, 0.0);
                } else if (h < 3.0/6.0) {
                    rgb = vec3(0.0, c, x);
                } else if (h < 4.0/6.0) {
                    rgb = vec3(0.0, x, c);
                } else if (h < 5.0/6.0) {
                    rgb = vec3(x, 0.0, c);
                } else {
                    rgb = vec3(c, 0.0, x);
                }

                return vec4(rgb + vec3(m), a);
            }
        "#;
    let rgb2hsv = r#"
            vec4 rgb2hsv(float r, float g, float b, float a) {
                float cmax = max(r, max(g, b));
                float cmin = min(r, min(g, b));
                float delta = cmax - cmin;

                float h = 0.0;
                if (delta != 0.0) {
                    if (cmax == r) {
                        h = mod((g - b) / delta, 6.0) / 6.0;
                    } else if (cmax == g) {
                        h = ((b - r) / delta + 2.0) / 6.0;
                    } else if (cmax == b) {
                        h = ((r - g) / delta + 4.0) / 6.0;
                    }
                }
                float s = 0.0;
                if (cmax != 0.0) {
                    s = delta / cmax;
                }

                return vec4(h, s, cmax, a);
            }
        "#;

    let fragment_shader_source = match &gtype {
        GradientType::Gradient => {
            r#"
                uniform vec4 hue;
                in vec2 tex_coord;
                out vec4 out_color;
                void main() {
                    vec4 white = vec4(1.0, 1.0, 1.0, 1.0);
                    vec4 color = mix(white, hue, tex_coord.x);
                    out_color = color * tex_coord.y;
                }
            "#
        }
        GradientType::Slider(stype) => {
            let var = r#"
                    uniform vec4 color;
                    in vec2 tex_coord;
                    out vec4 out_color;
                "#;
            let func = match stype.as_str() {
                "r" => "void main() { out_color = vec4(tex_coord.x, color.g, color.b, 1.0); } ",
                "g" => "void main() { out_color = vec4(color.r, tex_coord.x, color.b, 1.0); } ",
                "b" => "void main() { out_color = vec4(color.r, color.g, tex_coord.x, 1.0); } ",
                "h" => {
                    "void main() {
                            out_color = hsv2rgb(tex_coord.x, 1.0, 1.0, 1.0);
                        } "
                }
                "s" => {
                    "void main() {
                            vec4 hsv = rgb2hsv(color.r, color.g, color.b, color.a);
                            out_color  = hsv2rgb(hsv.r, tex_coord.x, hsv.b, hsv.a);
                        } "
                }
                "v" => {
                    "void main() {
                            vec4 hsv = rgb2hsv(color.r, color.g, color.b, color.a);
                            out_color  = hsv2rgb(hsv.r, hsv.g, tex_coord.x, hsv.a);
                        } "
                }
                _ => "",
            };
            &format!("{hsv2rgb}\n{rgb2hsv}\n{var}\n{func}")
        }
    };
    (
        format!("{shader_version}\n{vertex_shader_source}"),
        format!("{shader_version}\n{fragment_shader_source}"),
    )
}
