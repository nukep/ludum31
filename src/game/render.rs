use util::png;
use opengl_util::shader::Program;
use opengl_util::texture::Texture2D;
use opengl_util::vertex::VertexArray;
use super::{Game, GameStepResult};

pub struct GameRenderState {
    tileset: Texture2D,
    tileset_vao: VertexArray,
    shader_program: Program
}

impl GameRenderState {
    pub fn new() -> GameRenderState {
        use opengl_util::shape;

        let shader_program = load_default_program();
        let a_position = shader_program.get_attrib("position");
        let a_texture_uv = shader_program.get_attrib("texture_uv");

        let tileset_data = include_bin!("../../assets/tileset.png");
        let tileset = match png::load_png32_data_and_upload(tileset_data) {
            Ok(tileset) => tileset,
            Err(e) => panic!("{}", e)
        };

        let tileset_vao = shape::gen_tileset(8, 8, a_position, a_texture_uv);

        GameRenderState {
            tileset: tileset,
            tileset_vao: tileset_vao,
            shader_program: shader_program
        }
    }

    pub fn render(&mut self, game: &Game, step_result: &GameStepResult) {
        use gl;
        use cgmath::{Matrix4, FixedArray};
        use util::matrix::MatrixBuilder;
        use std::num::Float;

        unsafe {
            let (w, h) = step_result.viewport;
            gl::Viewport(0, 0, w, h);
            gl::ClearColor(0.1, 0.1, 0.15, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::BLEND);
        };

        let u_projection_view = self.shader_program.get_uniform("projection_view");
        let u_model = self.shader_program.get_uniform("model");

        self.shader_program.use_program(|uniform| {
            uniform.set_mat4(u_projection_view, step_result.projection_view.as_fixed());
            unsafe {
                gl::Enable(gl::TEXTURE_2D);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            };
            self.tileset.bind(0);
            self.tileset_vao.bind_vao(|vao_ctx| {
                let draw_tile = |x: f32, y: f32, id: u16, flip: (bool, bool)| {
                    let (flip_x, flip_y) = flip;

                    let mut model = Matrix4::identity()
                        .translate(x, y, 0.0)
                        .scale(16.0, 16.0, 0.0);
                    if flip_x {
                        model = model
                            .translate(1.0, 0.0, 0.0)
                            .scale(-1.0, 1.0, 1.0);
                    }
                    if flip_y {
                        model = model
                            .translate(0.0, 1.0, 0.0)
                            .scale(1.0, -1.0, 1.0);
                    }
                    uniform.set_mat4(u_model, model.as_fixed());
                    vao_ctx.draw_arrays(gl::TRIANGLES, (6*id) as i32, 6);
                };
                let tile_size = 16.0;
                let (width, height) = (game.level.width as f32 * tile_size, game.level.height as f32 * tile_size);

                let draw_tile_all = |x: f32, y: f32, id: u16, flip: (bool, bool)| {
                    // FIXME
                    draw_tile(x - width, y - height, id, flip);
                    draw_tile(x, y - height, id, flip);
                    draw_tile(x + width, y - height, id, flip);
                    draw_tile(x - width, y, id, flip);
                    draw_tile(x, y, id, flip);
                    draw_tile(x + width, y, id, flip);
                    draw_tile(x - width, y + height, id, flip);
                    draw_tile(x, y + height, id, flip);
                    draw_tile(x + width, y + height, id, flip);
                };

                // Draw all background tiles
                for (x, y, tile) in game.level.iter() {
                    if tile.tile_type.id > 0 {
                        let (fx, fy) = (x as f32 * tile_size, y as f32 * tile_size);
                        let id = tile.tile_type.id-1;
                        let flip = (tile.flip_x, tile.flip_y);

                        draw_tile_all(fx, fy, id, flip);
                    }
                }

                // Draw switches
                for switch in game.items.switches.iter() {
                    let tile = match switch.is_down {
                        false => 0x18,
                        true => 0x19
                    };
                    draw_tile_all(Float::floor(switch.x), Float::floor(switch.y), tile, (false, false));
                }

                // Draw beanstalks
                for beanstalk in game.items.beanstalks.iter().filter(|t| t.visible) {
                    for y in range(0, beanstalk.height) {
                        let tile = match y % 2 {
                            0 => 0x0E,
                            _ => 0x0F
                        };
                        draw_tile_all(Float::floor(beanstalk.x), Float::floor(beanstalk.y + y as f32 * 16.0), tile, (false, false));
                    }
                }

                // Draw chests
                for chest in game.items.chests.iter().filter(|t| t.visible) {
                    let tile_offset = match chest.is_static {
                        true => 0,
                        false => 5
                    };
                    let tile = match chest.phase*5.0 {
                        0.0...1.0 => 0x04,
                        1.0...2.0 => 0x05,
                        2.0...3.0 => 0x06,
                        3.0...4.0 => 0x07,
                        4.0...5.0 => 0x08,
                        _ => 0x08
                    } + tile_offset;
                    draw_tile_all(Float::floor(chest.x), Float::floor(chest.y)+3.0, tile, (false, false));
                }

                // Draw player
                {
                    use super::player::{PlayerState, PlayerItem};

                    match game.player.state {
                        PlayerState::Stand(ref s) => {
                            let tile = match s.running_cycle {
                                None => 0,
                                Some(0.0...0.3) => 1,
                                Some(0.3...0.6) => 2,
                                Some(0.6...1.0) => 3,
                                _ => 3
                            };
                            draw_tile_all(Float::floor(s.x), Float::floor(s.y)+3.0, tile, s.direction.get_flip());
                            match game.player.item {
                                PlayerItem::None => (),
                                PlayerItem::Drill => {
                                    let tile = 0x23;
                                    let (flip_x, _) = s.direction.get_flip();
                                    let x_offset = match flip_x {
                                        false => 12.0,
                                        true => -12.0
                                    };
                                    draw_tile_all(Float::floor(s.x)+x_offset, Float::floor(s.y)+3.0, tile, s.direction.get_flip());
                                },
                                PlayerItem::Gun => ()
                            }
                        },
                        PlayerState::Digging(ref s) => {
                            use super::player::PlayerDiggingDirection::{Up, Down, Left, Right};

                            let drill_tile: u16 = 0x23;

                            let (tile, flip_x) = match s.direction {
                                Up => (0x2D, false),
                                Down => (0x37, false),
                                Left => (0x38, true),
                                Right => (0x38, false)
                            };
                            draw_tile_all(Float::floor(s.x), Float::floor(s.y), tile, (flip_x, false));
                        },
                        PlayerState::Emerging(ref s) => {
                            let tile = 0x3A;
                            draw_tile_all(Float::floor(s.x), Float::floor(s.y), tile, (true, false));
                        }
                    };
                }

                // Draw monsters
                for monster1 in game.items.monsters1.iter().filter(|m| m.visible ) {
                    let tile = match monster1.phase * 2.0 {
                        0.0...1.0 => 0x26,
                        0.0...2.0 => 0x27,
                        _ => 0x27
                    };
                    draw_tile_all(Float::floor(monster1.x), Float::floor(monster1.y), tile, (false, false));
                }

                for monster2 in game.items.monsters2.iter().filter(|m| m.visible ) {
                    let tile = match monster2.phase * 2.0 {
                        0.0...1.0 => 0x28,
                        0.0...2.0 => 0x29,
                        _ => 0x27
                    };
                    draw_tile_all(Float::floor(monster2.x), Float::floor(monster2.y), tile, (false, false));
                }

                // Draw poofs
                for poof in game.items.poofs.iter() {
                    let tile = match poof.phase*6.0 {
                        0.0...1.0 => 0x1A,
                        1.0...2.0 => 0x1B,
                        2.0...3.0 => 0x1C,
                        3.0...4.0 => 0x1D,
                        4.0...5.0 => 0x1E,
                        5.0...6.0 => 0x1F,
                        _ => 0x1F
                    };
                    draw_tile_all(Float::floor(poof.x), Float::floor(poof.y), tile, (false, false));
                }

                // match game.player.state { }
            });
        });
    }
}


fn load_default_program() -> Program {
    use opengl_util::shader::{Shader};

    let vertex_source = include_str!("shaders/vertex.glsl");
    let fragment_source = include_str!("shaders/fragment.glsl");

    let vertex = match Shader::vertex_from_source(vertex_source) {
        Ok(shader) => shader,
        Err(s) => panic!("Vertex shader compilation error: {}", s)
    };
    let fragment = match Shader::fragment_from_source(fragment_source) {
        Ok(shader) => shader,
        Err(s) => panic!("Fragment shader compilation error: {}", s)
    };

    match Program::link("default".to_string(), &[&vertex, &fragment]) {
        Ok(program) => program,
        Err(s) => panic!("Shader link error: {}", s)
    }
}
