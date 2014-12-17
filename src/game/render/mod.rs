use util::png;
use opengl_util::shader::Program;
use opengl_util::texture::Texture2D;
use opengl_util::vertex::VertexArray;
use game_platforms::GameRenderer;
use super::{Game, GameStepResult};
use super::rect::Point;

mod tileset;

pub struct Renderer {
    tileset: Texture2D,
    tileset_vao: VertexArray,
    shader_program: Program
}

impl Renderer {
    pub fn new() -> Renderer {
        use opengl_util::shape;

        let shader_program = load_default_program();
        let a_position = shader_program.get_attrib("position");
        let a_texture_uv = shader_program.get_attrib("texture_uv");

        let tileset_data = include_bin!("../../../assets/tileset.png");
        let tileset = match png::load_png32_data_and_upload(tileset_data) {
            Ok(tileset) => tileset,
            Err(e) => panic!("{}", e)
        };

        let tileset_vao = shape::gen_tileset(8, 9, a_position, a_texture_uv);

        Renderer {
            tileset: tileset,
            tileset_vao: tileset_vao,
            shader_program: shader_program
        }
    }
}

impl GameRenderer<Game, GameStepResult> for Renderer {
    fn frame_limit(&self) -> Option<u32> { None }

    fn render(&mut self, game: &Game, step_result: &GameStepResult) {
        use gl;
        use cgmath::FixedArray;
        use std::num::Float;
        use self::tileset::TilesetDrawer;

        let screen_val = game.level.get_screen();
        let screen = &screen_val;

        unsafe {
            let (w, h) = step_result.viewport;
            gl::Viewport(0, 0, w, h);
            gl::Enable(gl::BLEND);
        };

        let u_projection_view = self.shader_program.get_uniform("projection_view");
        let u_model = self.shader_program.get_uniform("model");

        self.shader_program.use_program(|uniform| {
            unsafe {
                gl::Enable(gl::TEXTURE_2D);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            };
            self.tileset.bind(0);
            self.tileset_vao.bind_vao(|vao_ctx| {
                let tile_size = 16.0;
                let tileset_drawer = TilesetDrawer {
                    screen_size: (game.level.width as f32 * tile_size, game.level.height as f32 * tile_size),
                    tile_size: tile_size,
                    draw: |&: id, model| {
                        uniform.set_mat4(u_model, model);
                        vao_ctx.draw_arrays(gl::TRIANGLES, (6*id) as i32, 6);
                    }
                };
                let draw_tile_all = |xy: Point<f32>, id: u16, flip: (bool, bool), rotate_90: bool| {
                    let (x, y) = xy.floor(screen, 1.0).xy();
                    tileset_drawer.draw((x, y), id, flip, rotate_90);
                };

                uniform.set_mat4(u_projection_view, step_result.projection_view_parallax.as_fixed());
                // Draw parallax
                for y in range(0, game.level.height) {
                    for x in range(0, game.level.width) {
                        let pos = Point::new(screen, (x as f32 * tile_size, y as f32 * tile_size));
                        draw_tile_all(pos, 0x46, (false, false), false);
                    }
                }


                uniform.set_mat4(u_projection_view, step_result.projection_view.as_fixed());
                // Draw all background tiles
                for (x, y, tile) in game.level.iter() {
                    if tile.tile_type.id > 0 {
                        let f = (x as f32 * tile_size, y as f32 * tile_size);
                        let pos = Point::new(screen, f);
                        let id = tile.tile_type.id-1;
                        let flip = (tile.flip_x, tile.flip_y);

                        draw_tile_all(pos, id, flip, false);
                    }
                }

                // Draw messages
                for message in game.items.messages.iter().filter(|m| m.visible) {
                    for (i, tile_id) in message.tiles.iter().enumerate() {
                        let offset_x = i % message.width;
                        let offset_y = i / message.width;

                        let xy = message.xy.offset(screen, offset_x as f32 * tile_size, offset_y as f32 * tile_size);

                        draw_tile_all(xy, *tile_id, (false, false), false);
                    }
                }

                // Draw switches
                for switch in game.items.switches.iter().filter(|t| t.visible) {
                    let tile = match switch.is_down {
                        false => 0x18,
                        true => 0x19
                    };
                    draw_tile_all(switch.xy, tile, (false, false), false);
                }

                // Draw beanstalks
                for beanstalk in game.items.beanstalks.iter().filter(|t| t.visible) {
                    for y in range(0, beanstalk.height) {
                        let tile = match y % 2 {
                            0 => 0x0E,
                            _ => 0x0F
                        };
                        draw_tile_all(beanstalk.xy.offset(screen, 0.0, y as f32 * tile_size), tile, (false, false), false);
                    }
                }

                // Draw chests
                for chest in game.items.chests.iter().filter(|t| t.visible) {
                    let tile_offset = match chest.is_static {
                        true => 0,
                        false => 5
                    };
                    let tile = tile_from_phase(&[0x04, 0x05, 0x06, 0x07, 0x08], chest.phase) + tile_offset;
                    draw_tile_all(chest.xy.offset(screen, 0.0, tile_size * 3.0/16.0), tile, (false, false), false);
                }

                // Draw player
                {
                    use super::player::{PlayerState};

                    match game.player.state {
                        PlayerState::Stand(ref s) => {
                            let tile = if let Some(phase) = s.running_cycle {
                                tile_from_phase(&[0x01, 0x02, 0x03], phase)
                            } else {
                                0x00
                            };
                            draw_tile_all(s.xy.offset(screen, 0.0, tile_size * 3.0/16.0), tile, s.direction.get_flip(), false);
                            if let Some(_) = game.player.gun {
                                let tile = 0x3B;

                                let (flip_x, _) = s.direction.get_flip();
                                let x_offset = match flip_x {
                                    false => 4.0,
                                    true => -4.0
                                };
                                draw_tile_all(s.xy.offset(screen, x_offset, tile_size * 5.0/16.0), tile, s.direction.get_flip(), false);
                            } else if let Some(ref drill) = game.player.drill {
                                let tile = tile_from_phase(&[0x23, 0x24, 0x25, 0x36], drill.phase);

                                let (flip_x, _) = s.direction.get_flip();
                                let x_offset = match flip_x {
                                    false => 10.0,
                                    true => -10.0
                                };
                                draw_tile_all(s.xy.offset(screen, x_offset, tile_size * 3.0/16.0), tile, s.direction.get_flip(), false);
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

                            let drill_tile = if let Some(ref drill) = game.player.drill {
                                tile_from_phase(&[0x23, 0x24, 0x25, 0x36], drill.phase)
                            } else { panic!("Player is supposed to have drill!"); };

                            let (drill_behind, drill_flip, drill_rotate_90, drill_x, drill_y) = match s.direction {
                                Up => (true, (false, true), true, 0.0, -8.0),
                                Down => (false, (false, false), true, 0.0, 8.0),
                                Left => (false, (true, false), false, -10.0, 0.0),
                                Right => (false, (false, false), false, 10.0, 0.0)
                            };

                            if drill_behind {

                                draw_tile_all(s.xy.offset(screen, drill_x, drill_y), drill_tile, drill_flip, drill_rotate_90);
                                draw_tile_all(s.xy, tile, (flip_x, false), false);
                            } else {
                                draw_tile_all(s.xy, tile, (flip_x, false), false);
                                draw_tile_all(s.xy.offset(screen, drill_x, drill_y), drill_tile, drill_flip, drill_rotate_90);
                            }
                        },
                        PlayerState::Emerging(ref s) => {
                            let tile = 0x3A;
                            let flip_x = s.to_x < s.from_xy.x();
                            draw_tile_all(s.xy, tile, (flip_x, false), false);
                        },
                        PlayerState::Climbing(ref s) => {
                            let tile = 0x2D;
                            let flip_x = match s.phase*2.0 {
                                0.0...1.0 => false,
                                _ => true
                            };
                            draw_tile_all(s.xy, tile, (flip_x, false), false);
                        },
                        PlayerState::Dying(_) => ()
                    };
                }

                // Draw monsters
                for monster1 in game.items.monsters1.iter().filter(|m| m.visible ) {
                    let tile = tile_from_phase(&[0x26, 0x27], monster1.phase);
                    draw_tile_all(monster1.xy, tile, (false, false), false);
                }

                for monster2 in game.items.monsters2.iter().filter(|m| m.visible ) {
                    let tile = tile_from_phase(&[0x28, 0x29], monster2.phase);
                    draw_tile_all(monster2.xy, tile, (false, false), false);
                }

                // Draw keys
                for key in game.items.keys.iter().filter(|k| k.visible) {
                    let tile = match key.is_sticky {
                        true => 0x2E,
                        false => 0x2F
                    };
                    draw_tile_all(key.xy, tile, (false, false), false);
                }

                // Draw useless
                for useless in game.items.useless.iter() {
                    let tile = 0x45;
                    draw_tile_all(useless.xy, tile, (false, false), false);
                }

                // Draw poofs
                for poof in game.items.poofs.iter() {
                    let tile = tile_from_phase(&[0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F], poof.phase);
                    draw_tile_all(poof.xy, tile, (false, false), false);
                }

                // Draw bullets
                for bullet in game.items.bullets.iter() {
                    let tile = tile_from_phase(&[0x3C, 0x3D, 0x3E, 0x3F], bullet.phase);
                    let flip_x = bullet.vel_x < 0.0;
                    let offset_x = if flip_x { -16.0 } else { 0.0 };

                    draw_tile_all(bullet.xy.offset(screen, offset_x, -8.0), tile, (flip_x, false), false);
                }
            });
        });
    }
}

fn tile_from_phase(tiles: &[u16], phase: f32) -> u16 {
    let i = phase * tiles.len() as f32;
    let tile_index = if i < 0.0 { 0 } else if i >= tiles.len() as f32 { tiles.len() - 1 } else { i as uint };

    tiles[tile_index]
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
