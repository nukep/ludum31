use super::level::Tiles;
use super::wrapping::Screen;
use super::rect::{Point, Rect};

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

pub struct PlayerItemDrill {
    pub phase: f32
}

pub struct PlayerItemGun;

#[deriving(Copy)]
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

#[deriving(Copy)]
pub enum PlayerDiggingDirection {
    Up,
    Down,
    Left,
    Right
}

pub struct PlayerStateStand {
    pub direction: PlayerStandDirection,
    pub xy: Point<f32>,
    pub running_cycle: Option<f32>,
    vel_x: f32,
    vel_y: f32,
}

impl PlayerStateStand {
    fn apply_gravity(&mut self, screen: &Screen, tiles: &Tiles) {
        let vel_y = {
            let vy = self.vel_y;
            if self.go(screen, tiles, 0.0, vy) {
                0.0
            } else {
                vy + 0.5
            }
        };
        self.vel_y = if vel_y > 10.0 { 10.0 }
        else { vel_y };
    }

    fn run(&mut self, screen: &Screen, tiles: &Tiles, left: bool, right: bool) {
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

        self.go(screen, tiles, vel_x, 0.0);

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

    fn go(&mut self, screen: &Screen, tiles: &Tiles, x_delta: f32, y_delta: f32) -> bool {
        self.xy = self.xy.offset(screen, x_delta, y_delta);

        let direction = into_direction(y_delta < 0.0, y_delta > 0.0, x_delta < 0.0, x_delta > 0.0);

        match tiles.collision_tile(&self.get_rect(), direction) {
            Some((x, y)) => {
                self.xy = Point::new(screen, (x, y));
                true
            },
            None => false
        }
    }

    fn get_rect(&self) -> Rect<f32> {
        Rect::new(self.xy, (16.0, 16.0))
    }
}

pub struct PlayerStateDigging {
    pub direction: PlayerDiggingDirection,
    pub xy: Point<f32>
}

impl PlayerStateDigging {
    fn dig(&mut self, screen: &Screen, tiles: &Tiles, up: bool, down: bool, left: bool, right: bool) -> Option<PlayerState> {
        let speed = 2.0;
        let (xy, direction) = if up {
            self.direction = PlayerDiggingDirection::Up;
            (self.xy.offset(screen, 0.0, -speed), Some(into_direction(true, false, false, false)))
        } else if down {
            self.direction = PlayerDiggingDirection::Down;
            (self.xy.offset(screen, 0.0, speed), Some(into_direction(false, true, false, false)))
        } else if left {
            self.direction = PlayerDiggingDirection::Left;
            (self.xy.offset(screen, -speed, 0.0), Some(into_direction(false, false, true, false)))
        } else if right {
            self.direction = PlayerDiggingDirection::Right;
            (self.xy.offset(screen, speed, 0.0), Some(into_direction(false, false, false, true)))
        } else {
            (self.xy, None)
        };

        self.xy = xy;

        match direction {
            Some(direction) => {
                match tiles.collision_tile_digging(&self.get_rect(), direction, up) {
                    Some((x, y, emerge_hit)) => {
                        if emerge_hit {
                            Some(PlayerState::Emerging(PlayerStateEmerging::new(xy, x - 16.0, y - 16.0)))
                        } else {
                            self.xy = Point::new(screen, (x, y));
                            None
                        }
                    },
                    None => None
                }
            },
            None => None
        }
    }

    fn get_rect(&self) -> Rect<f32> {
        Rect::new(self.xy, (16.0, 16.0))
    }
}

/// When the player comes out of the dirt
/// Digging -> Emerging -> Stand
pub struct PlayerStateEmerging {
    pub xy: Point<f32>,
    pub from_xy: Point<f32>,
    pub to_x: f32,
    pub to_y: f32,

    phase: f32
}

impl PlayerStateEmerging {
    pub fn new(from_xy: Point<f32>, to_x: f32, to_y: f32) -> PlayerStateEmerging {
        PlayerStateEmerging {
            xy: from_xy,
            from_xy: from_xy,
            to_x: to_x,
            to_y: to_y,
            phase: 0.0
        }
    }

    pub fn tick(&mut self, screen: &Screen) -> Option<PlayerState> {
        self.phase += 0.04;
        if self.phase >= 1.0 {
            let direction = if self.to_x < self.from_xy.x() {
                PlayerStandDirection::Left
            } else {
                PlayerStandDirection::Right
            };

            Some(PlayerState::Stand(PlayerStateStand {
                direction: direction,
                xy: Point::new(screen, (self.to_x, self.to_y)),
                vel_x: 0.0,
                vel_y: 0.0,
                running_cycle: None
            }))
        } else {
            use std::num::{Float, FloatMath};

            fn lerp(a: f32, b: f32, p: f32) -> f32 { (b-a)*p + a }

            let coeff = 2.3;
            let y_phase = FloatMath::sin(self.phase * coeff) / FloatMath::sin(coeff);
            let x_phase = self.phase.powf(3.0);

            let (fx, fy) = self.from_xy.xy();

            let new_coord = (lerp(fx, self.to_x, x_phase), lerp(fy, self.to_y, y_phase));
            self.xy = Point::new(screen, new_coord);

            None
        }
    }
}

pub struct PlayerStateClimbing {
    pub xy: Point<f32>,
    pub phase: f32,

    beanstalk_y: f32,
    beanstalk_y_max: f32
}

impl PlayerStateClimbing {
    fn climb_up(&mut self, screen: &Screen) {
        let y = self.xy.y() - 2.0;
        self.phase = (self.phase + 0.1) % 1.0;

        let ny = if y < self.beanstalk_y { self.beanstalk_y }
        else { y };

        self.xy = self.xy.set_y(screen, ny);
    }

    fn climb_down(&mut self, screen: &Screen) {
        let y = self.xy.y() + 2.0;
        self.phase = (self.phase + 0.1) % 1.0;

        let ny = if y > self.beanstalk_y_max { self.beanstalk_y_max }
        else { y };

        self.xy = self.xy.set_y(screen, ny);
    }

    fn get_rect(&self) -> Rect<f32> {
        Rect::new(self.xy, (16.0, 16.0))
    }
}

pub struct PlayerStateDying {
    pub xy: Point<f32>,
    pub phase: f32,

    regen_coord: Point<f32>
}

pub enum PlayerState {
    Stand(PlayerStateStand),
    Digging(PlayerStateDigging),
    Emerging(PlayerStateEmerging),
    Climbing(PlayerStateClimbing),
    Dying(PlayerStateDying)
}

pub struct Player {
    pub state: PlayerState,
    pub drill: Option<PlayerItemDrill>,
    pub gun: Option<PlayerItemGun>,
    pub keys: uint
}

impl Player {
    pub fn new(pos: Point<f32>) -> Player {
        Player {
            state: Player::get_initial_state(pos),
            drill: None,
            gun: None,
            keys: 0
        }
    }

    fn get_initial_state(xy: Point<f32>) -> PlayerState {
        PlayerState::Stand(PlayerStateStand {
            direction: PlayerStandDirection::Left,
            xy: xy,
            vel_x: 0.0,
            vel_y: 0.0,
            running_cycle: None
        })
    }

    pub fn tick(&mut self, screen: &Screen, tiles: &Tiles, up: bool, down: bool, left: bool, right: bool) {
        let next_state: Option<PlayerState> = match self.state {
            PlayerState::Stand(ref mut s) => {
                s.apply_gravity(screen, tiles);
                s.run(screen, tiles, left, right);

                let has_drill = if let Some(_) = self.drill { true } else { false };

                if has_drill && down {
                    match tiles.is_dirt_entrance_below(&s.get_rect()) {
                        Some((x, y)) => {
                            // Dig it up!
                            Some(PlayerState::Digging(PlayerStateDigging {
                                direction: PlayerDiggingDirection::Down,
                                xy: Point::new(screen, (x as f32 * 16.0, y as f32 * 16.0))
                            }))
                        },
                        None => None
                    }
                } else {
                    None
                }
            },
            PlayerState::Digging(ref mut s) => {
                s.dig(screen, tiles, up, down, left, right)
            },
            PlayerState::Emerging(ref mut s) => {
                s.tick(screen)
            },
            PlayerState::Climbing(ref mut s) => {
                if up {
                    s.climb_up(screen);
                    None
                } else if down {
                    s.climb_down(screen);
                    None
                } else if left {
                    // Try to jump off
                    let rect = s.get_rect().offset(screen, -16.0, 0.0);
                    match tiles.has_non_blocking_tile(&rect) {
                        Some((x, y)) => {
                            Some(PlayerState::Emerging(PlayerStateEmerging::new(s.xy, x as f32 * 16.0, y as f32 * 16.0)))
                        },
                        None => None
                    }
                } else if right {
                    // Try to jump off
                    let rect = s.get_rect().offset(screen, 16.0, 0.0);
                    match tiles.has_non_blocking_tile(&rect) {
                        Some((x, y)) => {
                            Some(PlayerState::Emerging(PlayerStateEmerging::new(s.xy, x as f32 * 16.0, y as f32 * 16.0)))
                        },
                        None => None
                    }
                } else {
                    None
                }
            },
            PlayerState::Dying(ref mut s) => {
                let phase = s.phase + 0.05;
                if phase >= 1.0 {
                    Some(Player::get_initial_state(s.regen_coord))
                } else {
                    s.phase = phase;
                    None
                }
            }
        };

        self.tick_item();

        match next_state {
            Some(s) => self.state = s,
            None => ()
        }
    }

    fn tick_item(&mut self) {
        if let Some(ref mut drill) = self.drill {
            drill.phase = (drill.phase + 0.1) % 1.0;
        }
    }

    pub fn get_pos(&self) -> Point<f32> {
        match self.state {
            PlayerState::Stand(ref s) => s.xy,
            PlayerState::Digging(ref s) => s.xy,
            PlayerState::Emerging(ref s) => s.xy,
            PlayerState::Climbing(ref s) => s.xy,
            PlayerState::Dying(ref s) => s.xy
        }
    }

    pub fn get_rect(&self) -> Rect<f32> {
        Rect::new(self.get_pos(), (16.0, 16.0))
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

    pub fn is_drilling(&self) -> bool {
        match self.state {
            PlayerState::Digging(_) => {
                true
            },
            _ => false
        }
    }

    pub fn is_jumping(&self) -> bool {
        match self.state {
            PlayerState::Emerging(_) => {
                true
            },
            _ => false
        }
    }

    pub fn add_drill(&mut self) {
        self.drill = Some(PlayerItemDrill {
            phase: 0.0
        });
    }

    pub fn add_gun(&mut self) {
        self.gun = Some(PlayerItemGun);
    }

    pub fn try_climb_beanstalk(&mut self, screen: &Screen, beanstalk: &Rect<f32>) {
        let next_state: Option<PlayerState> = match self.state {
            PlayerState::Stand(ref _s) => {
                Some(PlayerState::Climbing(PlayerStateClimbing {
                    xy: self.get_pos().set_x(screen, beanstalk.x()),
                    beanstalk_y: beanstalk.y(),
                    beanstalk_y_max: beanstalk.y() + beanstalk.height() - 16.0,
                    phase: 0.0
                }))
            },
            _ => None
        };

        match next_state {
            Some(s) => self.state = s,
            None => ()
        }
    }

    pub fn die(&mut self, regen_coord: Point<f32>) {
        self.state = PlayerState::Dying(PlayerStateDying {
            xy: self.get_pos(),
            regen_coord: regen_coord,
            phase: 0.0
        });
    }

    pub fn is_alive(&self) -> bool {
        if let PlayerState::Dying(_) = self.state {
            false
        } else {
            true
        }
    }

    pub fn add_keys(&mut self, keys: uint) {
        self.keys += keys;
    }

    pub fn try_use_key(&mut self) -> bool {
        if self.keys > 0 {
            self.keys -= 1;
            true
        } else {
            false
        }
    }
}
