use sdl2;

use cgmath;
use game_platforms::{PlatformStepResult, GameStepper, GameRenderer};
use game_platforms::sdl2_opengl::{Input, RenderContext};
use self::audio::Audio;
use self::items::DynamicItems;
use self::level::Level;
use self::render::{GameRenderState};
use self::player::Player;

mod audio;
mod collision;
mod items;
mod level;
mod rect;
mod player;
pub mod render;

pub struct Game {
    audio: Option<Audio>,
    pub level: Level,
    pub items: DynamicItems,
    player: Player,
    scroll_x: f32,
    scroll_y: f32
}

pub struct GameStepResult {
    viewport: (i32, i32),
    projection_view: cgmath::Matrix4<f32>
}

impl Game {
    pub fn new() -> Game {
        let audio = match Audio::new() {
            Ok(audio) => Some(audio),
            Err(e) => {
                println!("{}", e);
                println!("Audio will be disabled");
                None
            }
        };
        let level = Level::load();
        let items = DynamicItems::new(&level);
        let player = Player::new(level.player_start_pos);
        let scroll_x = 0.0;

        Game {
            audio: audio,
            level: level,
            items: items,
            player: player,
            scroll_x: scroll_x,
            scroll_y: 0.0
        }
    }

    fn scroll(&mut self, x: f32, y: f32) {
        let scroll_x = self.scroll_x + x;
        let scroll_y = self.scroll_y + y;
        let (nx, ny) = self.level.wrap_coordinates((scroll_x, scroll_y));
        self.scroll_x = nx;
        self.scroll_y = ny;
    }
}

impl GameStepper<Input, GameStepResult> for Game {
    fn steps_per_second() -> u32 { 60 }
    fn step(&mut self, input: &Input) -> PlatformStepResult<GameStepResult> {
        use game_platforms::PlatformStepResult::{Continue, Exit};
        use sdl2::keycode::KeyCode;
        use cgmath::ToMatrix4;

        if input.exit_request() {
            return Exit;
        }

        let lock_scrolling = input.is_keycode_down(KeyCode::LCtrl) | input.is_keycode_down(KeyCode::RCtrl);
        let up = input.is_keycode_down(KeyCode::Up) | input.is_keycode_down(KeyCode::W);
        let down = input.is_keycode_down(KeyCode::Down) | input.is_keycode_down(KeyCode::S);
        let left = input.is_keycode_down(KeyCode::Left) | input.is_keycode_down(KeyCode::A);
        let right = input.is_keycode_down(KeyCode::Right) | input.is_keycode_down(KeyCode::D);

        let last_player_pos = self.player.get_pos();
        let last_player_is_walking = self.player.is_walking();
        self.player.tick(&self.level, up, down, left, right);
        let cur_player_pos = self.player.get_pos();

        let got_item = if down {
            let items = self.items.try_open_chest(self.player.get_rect());
            let (px, py) = cur_player_pos;

            for item in items.iter() {
                use self::items::ChestItem;
                use self::player::PlayerItem;
                match item {
                    &ChestItem::Drill => {
                        self.player.item = PlayerItem::Drill;
                        self.items.add_poof(px+5.0, py+5.0);
                    },
                    &ChestItem::Gun => {
                        self.player.item = PlayerItem::Gun;
                    },
                    _ => ()
                }
            }

            items.len() > 0
        } else {
            false
        };

        self.items.step_poofs();
        self.items.step_chests();
        self.items.step_monsters();
        {
            let switch_triggers: Vec<u8> = {
                let switches = match cur_player_pos {
                    (x, y) => self.items.switch_hit_test(x, y, 16.0, 16.0)
                };

                switches.iter().map(|switch| {
                    switch.trigger
                }).collect()
            };
            let mut play_poof_sound = false;

            for trigger in switch_triggers.iter() {
                play_poof_sound |= self.items.trigger(*trigger);
            }

            let cur_player_is_walking = self.player.is_walking();

            let (moved, destroyed) = if !lock_scrolling {
                let (lx, ly) = last_player_pos;
                let (cx, cy) = cur_player_pos;
                let (rel_x, rel_y) = self.level.relative_wrap(last_player_pos, cur_player_pos);

                match (rel_x, rel_y) {
                    (0.0, 0.0) => (false, false),
                    (sx, sy) => {
                        self.scroll(sx, sy);
                        self.items.adjust_to_scroll_boundary(&self.level, self.scroll_x, self.scroll_y, rel_x > 0.0, rel_y > 0.0, rel_x < 0.0, rel_y < 0.0)
                    }
                }
            } else {
                (false, false)
            };

            if let Some(ref mut audio) = self.audio {
                match (last_player_is_walking, cur_player_is_walking) {
                    (false, true) => audio.start_walking(),
                    (true, false) => audio.stop_walking(),
                    _ => ()
                };

                if destroyed {
                    audio.explode();
                }

                if play_poof_sound {
                    audio.poof();
                }

                if got_item {
                    audio.item_get();
                    audio.poof();
                }
            }
        }

        let projection_view = cgmath::ortho(
            0.0 + self.scroll_x,
            self.level.width as f32 * 16.0 + self.scroll_x,
            self.level.height as f32 * 16.0 + self.scroll_y,
            0.0 + self.scroll_y,
            -1.0,
            1.0
        );

        Continue(GameStepResult {
            viewport: input.get_viewport(),
            projection_view: projection_view
        })
    }
}

impl GameRenderer<RenderContext<(), GameRenderState>, GameStepResult> for Game {
    fn render(&self, step_result: &GameStepResult, ctx: &mut RenderContext<(), GameRenderState>) {
        ctx.state.render(self, step_result);
    }

    fn frame_limit(&self) -> Option<u32> { None }
}
