use sdl2;

use cgmath;
use game_platforms::{PlatformStepResult, GameStepper};
use game_platforms::sdl2_opengl::Input;
use self::audio::Audio;
use self::items::DynamicItems;
use self::level::Level;
use self::player::Player;
use self::rect::RectExt;

mod audio;
mod collision;
mod items;
mod level;
mod rect;
mod player;
mod wrapping;
pub mod render;

pub struct Game {
    audio: Option<Audio>,
    pub level: Level,
    pub items: DynamicItems,
    player: Player,
    scroll_x: f32,
    scroll_y: f32,
    exited: bool
}

pub struct GameStepResult {
    viewport: (i32, i32),
    projection_view: cgmath::Matrix4<f32>,
    projection_view_parallax: cgmath::Matrix4<f32>,
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
            scroll_y: 0.0,
            exited: false
        }
    }

    fn scroll(&mut self, x: f32, y: f32) {
        let scroll_x = self.scroll_x + x;
        let scroll_y = self.scroll_y + y;
        let (nx, ny) = self.level.get_screen().wrap_coord((scroll_x, scroll_y));
        self.scroll_x = nx;
        self.scroll_y = ny;
    }
}

impl GameStepper<Input, GameStepResult> for Game {
    fn steps_per_second() -> u32 { 60 }
    fn step(&mut self, input: &Input) -> PlatformStepResult<GameStepResult> {
        use game_platforms::PlatformStepResult::{Continue, Exit};
        use sdl2::keycode::KeyCode;

        if input.exit_request() {
            return Exit;
        }

        let lock_scrolling = input.is_keycode_down(KeyCode::LCtrl) | input.is_keycode_down(KeyCode::RCtrl) | input.is_keycode_down(KeyCode::ScrollLock) | !self.player.is_alive();
        let fire = input.is_keycode_newly_down(KeyCode::Space) | input.is_mouse_button_newly_down(sdl2::mouse::LEFTMOUSESTATE);
        let up = input.is_keycode_down(KeyCode::Up) | input.is_keycode_down(KeyCode::W);
        let down = input.is_keycode_down(KeyCode::Down) | input.is_keycode_down(KeyCode::S);
        let new_down = input.is_keycode_newly_down(KeyCode::Down) | input.is_keycode_newly_down(KeyCode::S);
        let left = input.is_keycode_down(KeyCode::Left) | input.is_keycode_down(KeyCode::A);
        let right = input.is_keycode_down(KeyCode::Right) | input.is_keycode_down(KeyCode::D);

        let last_player_pos = self.player.get_pos();
        let last_player_is_walking = self.player.is_walking();
        let last_player_is_drilling = self.player.is_drilling();
        let last_player_is_jumping = self.player.is_jumping();

        if up {
            use self::rect::RectExt;

            let player_rect = self.player.get_rect();
            let beanstalk = self.items.beanstalk_exists(player_rect);

            match beanstalk {
                Some(rect) => {
                    self.player.try_climb_beanstalk(rect.x(), player_rect.y(), rect.y(), rect.height());
                },
                None => ()
            }
        }

        let died = if self.player.is_alive() && self.items.rect_hits_monsters(self.player.get_rect()) {
            self.items.add_poof(last_player_pos.0, last_player_pos.1);
            true
        } else {
            false
        };

        if died {
            self.player.die(self.level.player_start_pos);
        }

        self.player.tick(&self.level.get_screen(), self.level.get_tiles(), up, down, left, right);
        let cur_player_pos = self.player.get_pos();
        let cur_player_is_walking = self.player.is_walking();
        let cur_player_is_drilling = self.player.is_drilling();
        let cur_player_is_jumping = self.player.is_jumping();
        let cur_player_rect = self.player.get_rect();

        let mut got_useless_points = false;

        let got_item = if new_down {
            let items = self.items.try_open_chest(cur_player_rect);
            let (px, py) = cur_player_pos;

            for item in items.iter() {
                use self::items::ChestItem;

                match item {
                    &(_, _, ChestItem::Drill) => {
                        self.player.add_drill();
                    },
                    &(_, _, ChestItem::Gun) => {
                        self.player.add_gun();
                    },
                    &(x, y, ChestItem::UselessPoints) => {
                        self.items.add_useless_points(x, y);
                        got_useless_points = true;
                    },
                    &(_, _, ChestItem::None) => ()
                }

                self.items.add_poof(px+5.0, py+5.0);
            }

            items.len() > 0
        } else {
            false
        };

        let used_key = if let Some((x, y)) = self.level.get_tiles().is_key_entrance_beside(cur_player_rect) {
            if self.player.try_use_key() {
                self.level.get_tiles_mut().remove_key_entrance(x, y);
                true
            } else {
                false
            }
        } else {
            false
        };

        let got_key = match self.items.try_take_keys(cur_player_rect) {
            0 => false,
            amount => {
                self.player.add_keys(amount);
                true
            }
        };

        let just_exited = if new_down {
            if let Some((_x, _y)) = self.level.get_tiles().is_tile_inside(cur_player_rect, 0x2B) {
                true
            } else {
                false
            }
        } else {
            false
        };

        let new_coins = {
            let screen = self.level.get_screen();
            let rect = cur_player_rect.set_width(4.0).offset(&screen, 6.0, 0.0);
            self.level.get_tiles_mut().take_coins(rect)
        };

        if just_exited {
            self.exited = true;
            self.items.trigger(255);
            self.level.trigger_set_to(255);
        }

        self.items.step(&self.level.get_screen());
        self.items.bullet_item_collision(self.level.get_tiles());

        let gun_fired = if fire {
            if let self::player::PlayerState::Stand(ref s) = self.player.state {
                if let Some(_) = self.player.gun {
                    use self::player::PlayerStandDirection::{Left, Right};

                    let (px, py) = cur_player_pos;
                    let (bullet_coord, vel_x) = match s.direction {
                        Left => ((px-4.0, py+12.0), -8.0),
                        Right => ((px+20.0, py+12.0), 8.0)
                    };
                    let new_coord = self.level.get_screen().wrap_coord(bullet_coord);
                    self.items.add_bullet(new_coord.0, new_coord.1, vel_x);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

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
                play_poof_sound |= self.level.trigger_set_to(*trigger);
            }

            let (_moved, destroyed) = if !lock_scrolling {
                let (rel_x, rel_y) = self.level.get_screen().relative_wrap(last_player_pos, cur_player_pos);

                match (rel_x, rel_y) {
                    (0.0, 0.0) => (false, false),
                    (sx, sy) => {
                        if self.exited {
                            self.scroll(sx, sy);
                        } else {
                            self.scroll(sx, 0.0);
                        }
                        self.items.adjust_to_scroll_boundary(&self.level.get_screen(), self.level.get_tiles(), self.scroll_x, rel_x > 0.0, rel_x < 0.0)
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

                match (last_player_is_drilling, cur_player_is_drilling) {
                    (false, true) => audio.start_drilling(),
                    (true, false) => audio.stop_drilling(),
                    _ => ()
                };

                match (last_player_is_jumping, cur_player_is_jumping) {
                    (false, true) => audio.jump(),
                    _ => ()
                };

                if destroyed { audio.explode(); }

                if play_poof_sound { audio.poof(); }

                if got_item { audio.item_get(); }

                if got_useless_points { audio.nothing(); }

                if got_key { audio.key_get(); }

                if used_key { audio.unlock(); }

                if gun_fired { audio.fire(); }

                if new_coins > 0 {
                    audio.coin();
                }

                if died { audio.die(); }
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

        let parallax_rate = 0.5;

        let projection_view_parallax = cgmath::ortho(
            0.0 + self.scroll_x * parallax_rate,
            self.level.width as f32 * 16.0 + self.scroll_x * parallax_rate,
            self.level.height as f32 * 16.0 + self.scroll_y * parallax_rate,
            0.0 + self.scroll_y * parallax_rate,
            -1.0,
            1.0
        );

        Continue(GameStepResult {
            viewport: input.get_viewport(),
            projection_view: projection_view,
            projection_view_parallax: projection_view_parallax
        })
    }
}
