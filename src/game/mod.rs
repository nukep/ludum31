use sdl2;

use cgmath;
use game_platforms::{PlatformStepResult, GameStepper, GameRenderer};
use game_platforms::sdl2_opengl::{Input, RenderContext};
use self::audio::Audio;
use self::items::DynamicItems;
use self::level::Level;
use self::render::{GameRenderState};

mod audio;
mod collision;
mod items;
mod level;
pub mod render;

fn into_direction(up: bool, down: bool, left: bool, right: bool) -> (Option<bool>, Option<bool>){
    let direction_down = match (up, down) {
        (false, false) => None,
        (true, false) => Some(false),
        (false, true) => Some(true),
        (true, true) => Some(true)
    };

    let direction_right = match (left, right) {
        (false, false) => None,
        (true, false) => Some(false),
        (false, true) => Some(true),
        (true, true) => Some(true)
    };

    (direction_right, direction_down)
}

pub enum PlayerStandDirection {
    Left,
    Right
}

impl PlayerStandDirection {
    pub fn get_flip(&self) -> (bool, bool) {
        match self {
            &PlayerStandDirection::Left => (true, false),
            &PlayerStandDirection::Right => (false, false)
        }
    }
}

pub enum PlayerDiggingDirection {
    Up,
    Down,
    Left,
    Right
}

pub struct PlayerStateStand {
    direction: PlayerStandDirection,
    x: f32,
    y: f32,
    vel_x: f32,
    vel_y: f32,
    running_cycle: Option<f32>
}

impl PlayerStateStand {
    fn apply_gravity(&mut self, level: &Level) {
        let vel_y = {
            let vy = self.vel_y;
            if self.go(level, 0.0, vy) {
                0.0
            } else {
                self.vel_y + 0.5
            }
        };
        self.vel_y = vel_y;
    }

    fn run(&mut self, level: &Level, left: bool, right: bool) {
        use std::num::Float;

        let speed_increment = 0.25;
        let speed_slowdown = 0.8;

        let vel_x = {
            let vx = if left {
                self.vel_x - speed_increment
            } else if right {
                self.vel_x + speed_increment
            } else {
                self.vel_x * speed_slowdown
            };

            let max_speed = 3.0;

            if vx > max_speed { max_speed }
            else if vx < -max_speed { -max_speed }
            else { vx }
        };

        self.go(level, vel_x, 0.0);

        self.vel_x = vel_x;

        self.direction = if left { PlayerStandDirection::Left }
            else if right { PlayerStandDirection::Right }
            else { self.direction };

        self.running_cycle = if Float::abs(vel_x) > 0.2 {
            match self.running_cycle {
                None => Some(0.0),
                Some(v) => Some((v + Float::abs(vel_x) * 0.04) % 1.0)
            }
        } else {
            None
        };
    }

    fn go(&mut self, level: &Level, x_delta: f32, y_delta: f32) -> bool {
        let new_coord = level.wrap_coordinates((self.x + x_delta, self.y + y_delta));
        self.x = new_coord.val0();
        self.y = new_coord.val1();

        let direction = into_direction(y_delta < 0.0, y_delta > 0.0, x_delta < 0.0, x_delta > 0.0);

        match level.collision_tile((self.x, self.y), direction) {
            Some((x, y)) => {
                self.x = x;
                self.y = y;
                true
            },
            None => false
        }
    }
}

pub enum PlayerState {
    Stand(PlayerStateStand),
    Digging(PlayerDiggingDirection),
    Climbing(int)
}

pub struct Player {
    state: PlayerState
}

impl Player {
    pub fn new(pos: (f32, f32)) -> Player {
        let (x, y) = pos;
        Player {
            state: PlayerState::Stand(PlayerStateStand {
                direction: PlayerStandDirection::Left,
                x: x,
                y: y,
                vel_x: 0.0,
                vel_y: 0.0,
                running_cycle: None
            })
        }
    }

    pub fn tick(&mut self, level: &Level, up: bool, down: bool, left: bool, right: bool) {
        match self.state {
            PlayerState::Stand(ref mut s) => {
                s.apply_gravity(level);
                s.run(level, left, right);
            },
            _ => ()
        };
    }

    pub fn get_pos(&self) -> (f32, f32) {
        use std::num::Float;

        match self.state {
            PlayerState::Stand(ref s) => {
                (Float::floor(s.x), Float::floor(s.y))
            },
            _ => panic!("Unimplemented")
        }
    }

    pub fn is_walking(&self) -> bool {
        use std::num::Float;

        match self.state {
            PlayerState::Stand(ref s) => {
                Float::abs(s.vel_x) > 0.5
            },
            _ => false
        }
    }
}

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
        self.items.step_poofs();
        self.items.step_falling_chests();
        {
            let cur_player_pos = self.player.get_pos();
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

            if let Some(ref mut audio) = self.audio {
                match (last_player_is_walking, cur_player_is_walking) {
                    (false, true) => audio.start_walking(),
                    (true, false) => audio.stop_walking(),
                    _ => ()
                };

                if play_poof_sound {
                    audio.poof();
                }
            }

            if !lock_scrolling {
                let (lx, ly) = last_player_pos;
                let (cx, cy) = cur_player_pos;
                self.scroll(cx-lx, cy-ly);
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
