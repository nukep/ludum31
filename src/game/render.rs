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
                let draw_tile = |x: f32, y: f32, id: u16, flip: (bool, bool), rotate_90: bool| {
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
                    if rotate_90 {
                        use std::f32::consts::FRAC_PI_2;
                        model = model
                        .translate(1.0, 0.0, 0.0)
                        .rotate_z(FRAC_PI_2);
                    }
                    uniform.set_mat4(u_model, model.as_fixed());
                    vao_ctx.draw_arrays(gl::TRIANGLES, (6*id) as i32, 6);
                };
                let tile_size = 16.0;
                let (width, height) = (game.level.width as f32 * tile_size, game.level.height as f32 * tile_size);

                let draw_tile_all = |x: f32, y: f32, id: u16, flip: (bool, bool), rotate_90: bool| {
                    // FIXME
                    draw_tile(x - width, y - height, id, flip, rotate_90);
                    draw_tile(x, y - height, id, flip, rotate_90);
                    draw_tile(x + width, y - height, id, flip, rotate_90);
                    draw_tile(x - width, y, id, flip, rotate_90);
                    draw_tile(x, y, id, flip, rotate_90);
                    draw_tile(x + width, y, id, flip, rotate_90);
                    draw_tile(x - width, y + height, id, flip, rotate_90);
                    draw_tile(x, y + height, id, flip, rotate_90);
                    draw_tile(x + width, y + height, id, flip, rotate_90);
                };

                // Draw all background tiles
                for (x, y, tile) in game.level.iter() {
                    if tile.tile_type.id > 0 {
                        let (fx, fy) = (x as f32 * tile_size, y as f32 * tile_size);
                        let id = tile.tile_type.id-1;
                        let flip = (tile.flip_x, tile.flip_y);

                        draw_tile_all(fx, fy, id, flip, false);
                    }
                }

                // Draw switches
                for switch in game.items.switches.iter() {
                    let tile = match switch.is_down {
                        false => 0x18,
                        true => 0x19
                    };
                    draw_tile_all(Float::floor(switch.x), Float::floor(switch.y), tile, (false, false), false);
                }

                // Draw beanstalks
                for beanstalk in game.items.beanstalks.iter().filter(|t| t.visible) {
                    for y in range(0, beanstalk.height) {
                        let tile = match y % 2 {
                            0 => 0x0E,
                            _ => 0x0F
                        };
                        draw_tile_all(Float::floor(beanstalk.x), Float::floor(beanstalk.y + y as f32 * 16.0), tile, (false, false), false);
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
                    draw_tile_all(Float::floor(chest.x), Float::floor(chest.y)+3.0, tile, (false, false), false);
                }

                // Draw player
                {
                    use super::player::{PlayerState};

                    match game.player.state {
                        PlayerState::Stand(ref s) => {
                            let tile = if let Some(phase) = s.running_cycle {
                                match phase * 3.0 {
                                    0.0...1.0 => 1,
                                    1.0...2.0 => 2,
                                    2.0...3.0 => 3,
                                    _ => 3
                                }
                            } else {
                                0
                            };
                            draw_tile_all(Float::floor(s.x), Float::floor(s.y)+3.0, tile, s.direction.get_flip(), false);
                            if let Some(_) = game.player.gun {
                                let tile = 0x3B;

                                let (flip_x, _) = s.direction.get_flip();
                                let x_offset = match flip_x {
                                    false => 4.0,
                                    true => -4.0
                                };
                                draw_tile_all(Float::floor(s.x)+x_offset, Float::floor(s.y)+5.0, tile, s.direction.get_flip(), false);
                            } else if let Some(ref drill) = game.player.drill {
                                let tile = match drill.phase * 4.0 {
                                    0.0...1.0 => 0x23,
                                    1.0...2.0 => 0x24,
                                    2.0...3.0 => 0x25,
                                    3.0...4.0 => 0x36,
                                    _ => 0x36
                                };

                                let (flip_x, _) = s.direction.get_flip();
                                let x_offset = match flip_x {
                                    false => 10.0,
                                    true => -10.0
                                };
                                draw_tile_all(Float::floor(s.x)+x_offset, Float::floor(s.y)+3.0, tile, s.direction.get_flip(), false);
                            }
                        },
                        PlayerState::Digging(ref s) => {
                            use super::player::PlayerDiggingDirection::{Up, Down, Left, Right};

                            let (tile, flip_x) = match s.direction {
                                Up => (0x2D, false),
                                Down => (0x37, false),
                                Left => (0x38, true),
                                Right => (0x38, false)
                            };

                            let drill_phase = if let Some(ref drill) = game.player.drill {
                                drill.phase
                            } else { panic!("Player is supposed to have drill!"); };

                            let drill_tile: u16 = match drill_phase * 4.0 {
                                0.0...1.0 => 0x23,
                                1.0...2.0 => 0x24,
                                2.0...3.0 => 0x25,
                                3.0...4.0 => 0x36,
                                _ => 0x36
                            };
                            let (drill_behind, drill_flip, drill_rotate_90, drill_x, drill_y) = match s.direction {
                                Up => (true, (false, true), true, 0.0, -8.0),
                                Down => (false, (false, false), true, 0.0, 8.0),
                                Left => (false, (true, false), false, -10.0, 0.0),
                                Right => (false, (false, false), false, 10.0, 0.0)
                            };

                            if drill_behind {
                                draw_tile_all(Float::floor(s.x) + drill_x, Float::floor(s.y) + drill_y, drill_tile, drill_flip, drill_rotate_90);
                                draw_tile_all(Float::floor(s.x), Float::floor(s.y), tile, (flip_x, false), false);
                            } else {
                                draw_tile_all(Float::floor(s.x), Float::floor(s.y), tile, (flip_x, false), false);
                                draw_tile_all(Float::floor(s.x) + drill_x, Float::floor(s.y) + drill_y, drill_tile, drill_flip, drill_rotate_90);
                            }
                        },
                        PlayerState::Emerging(ref s) => {
                            let tile = 0x3A;
                            let flip_x = s.to_x < s.from_x;
                            draw_tile_all(Float::floor(s.x), Float::floor(s.y), tile, (flip_x, false), false);
                        },
                        PlayerState::Climbing(ref s) => {
                            let tile = 0x2D;
                            let flip_x = match s.phase*2.0 {
                                0.0...1.0 => false,
                                _ => true
                            };
                            draw_tile_all(Float::floor(s.x), Float::floor(s.y), tile, (flip_x, false), false);
                        },
                        PlayerState::Dying(_) => ()
                    };
                }

                // Draw monsters
                for monster1 in game.items.monsters1.iter().filter(|m| m.visible ) {
                    let tile = match monster1.phase * 2.0 {
                        0.0...1.0 => 0x26,
                        0.0...2.0 => 0x27,
                        _ => 0x27
                    };
                    draw_tile_all(Float::floor(monster1.x), Float::floor(monster1.y), tile, (false, false), false);
                }

                for monster2 in game.items.monsters2.iter().filter(|m| m.visible ) {
                    let tile = match monster2.phase * 2.0 {
                        0.0...1.0 => 0x28,
                        0.0...2.0 => 0x29,
                        _ => 0x27
                    };
                    draw_tile_all(Float::floor(monster2.x), Float::floor(monster2.y), tile, (false, false), false);
                }

                // Draw keys
                for key in game.items.keys.iter().filter(|k| k.visible) {
                    let tile = match key.is_sticky {
                        true => 0x2E,
                        false => 0x2F
                    };
                    draw_tile_all(Float::floor(key.x), Float::floor(key.y), tile, (false, false), false);
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
                    draw_tile_all(Float::floor(poof.x), Float::floor(poof.y), tile, (false, false), false);
                }

                // Draw bullets
                for bullet in game.items.bullets.iter() {
                    let tile = match bullet.phase * 4.0 {
                        0.0...1.0 => 0x3C,
                        1.0...2.0 => 0x3D,
                        2.0...3.0 => 0x3E,
                        3.0...4.0 => 0x3F,
                        _ => 0x3F
                    };
                    let flip_x = bullet.vel_x < 0.0;
                    let offset_x = if flip_x { -16.0 } else { 0.0 };

                    draw_tile_all(Float::floor(bullet.x) + offset_x, Float::floor(bullet.y) - 8.0, tile, (flip_x, false), false);
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
