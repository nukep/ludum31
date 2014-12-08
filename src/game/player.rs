use super::level::Level;

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

pub enum PlayerItem {
    None,
    Drill(PlayerItemDrill),
    Gun
}

pub struct PlayerItemDrill {
    pub phase: f32
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
    pub direction: PlayerStandDirection,
    pub x: f32,
    pub y: f32,
    pub running_cycle: Option<f32>,
    vel_x: f32,
    vel_y: f32,
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

        match level.collision_tile(self.get_rect(), direction) {
            Some((x, y)) => {
                self.x = x;
                self.y = y;
                true
            },
            None => false
        }
    }

    fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.x + 16.0, self.y + 16.0)
    }
}

pub struct PlayerStateDigging {
    pub direction: PlayerDiggingDirection,
    pub x: f32,
    pub y: f32
}

impl PlayerStateDigging {
    fn dig(&mut self, level: &Level, up: bool, down: bool, left: bool, right: bool) -> Option<PlayerState> {
        let speed = 2.0;
        let (x, y, direction) = if up {
            self.direction = PlayerDiggingDirection::Up;
            (self.x, self.y - speed, Some(into_direction(true, false, false, false)))
        } else if down {
            self.direction = PlayerDiggingDirection::Down;
            (self.x, self.y + speed, Some(into_direction(false, true, false, false)))
        } else if left {
            self.direction = PlayerDiggingDirection::Left;
            (self.x - speed, self.y, Some(into_direction(false, false, true, false)))
        } else if right {
            self.direction = PlayerDiggingDirection::Right;
            (self.x + speed, self.y, Some(into_direction(false, false, false, true)))
        } else {
            (self.x, self.y, None)
        };

        let (nx, ny) = level.wrap_coordinates((x, y));
        self.x = nx;
        self.y = ny;

        match direction {
            Some(direction) => {
                match level.collision_tile_digging(self.get_rect(), direction, up) {
                    Some((x, y, emerge_hit)) => {
                        if emerge_hit {
                            Some(PlayerState::Emerging(PlayerStateEmerging {
                                from_x: nx,
                                from_y: ny,
                                to_x: x - 16.0,
                                to_y: y - 16.0,
                                x: nx,
                                y: ny,
                                phase: 0.0
                            }))
                        } else {
                            self.x = x;
                            self.y = y;
                            None
                        }
                    },
                    None => None
                }
            },
            None => None
        }
    }

    fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.x + 16.0, self.y + 16.0)
    }
}

/// When the player comes out of the dirt
/// Digging -> Emerging -> Stand
pub struct PlayerStateEmerging {
    pub x: f32,
    pub y: f32,

    from_x: f32,
    from_y: f32,
    to_x: f32,
    to_y: f32,
    phase: f32
}

impl PlayerStateEmerging {
    pub fn tick(&mut self) -> Option<PlayerState> {
        self.phase += 0.04;
        if self.phase >= 1.0 {
            Some(PlayerState::Stand(PlayerStateStand {
                direction: PlayerStandDirection::Left,
                x: self.to_x,
                y: self.to_y,
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

            self.x = lerp(self.from_x, self.to_x, x_phase);
            self.y = lerp(self.from_y, self.to_y, y_phase);
            None
        }
    }
}

pub enum PlayerState {
    Stand(PlayerStateStand),
    Digging(PlayerStateDigging),
    Emerging(PlayerStateEmerging)
}

pub struct Player {
    pub state: PlayerState,
    pub item: PlayerItem
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
            }),
            item: PlayerItem::None
        }
    }

    pub fn tick(&mut self, level: &Level, up: bool, down: bool, left: bool, right: bool) {
        let next_state: Option<PlayerState> = match self.state {
            PlayerState::Stand(ref mut s) => {
                s.apply_gravity(level);
                s.run(level, left, right);

                let has_drill = if let PlayerItem::Drill(_) = self.item { true } else { false };

                if has_drill && down {
                    match level.is_dirt_entrance_below(s.get_rect()) {
                        Some((x, y)) => {
                            // Dig it up!
                            Some(PlayerState::Digging(PlayerStateDigging {
                                direction: PlayerDiggingDirection::Down,
                                x: x as f32 * 16.0,
                                y: y as f32 * 16.0
                            }))
                        },
                        None => None
                    }
                } else {
                    None
                }
            },
            PlayerState::Digging(ref mut s) => {
                s.dig(level, up, down, left, right)
            },
            PlayerState::Emerging(ref mut s) => {
                s.tick()
            }
        };

        self.tick_item();

        match next_state {
            Some(s) => {
                self.state = s;
            },
            None => ()
        }
    }

    fn tick_item(&mut self) {
        match self.item {
            PlayerItem::Drill(ref mut drill) => {
                drill.phase = (drill.phase + 0.1) % 1.0;
            },
            _ => ()
        }
    }

    pub fn get_pos(&self) -> (f32, f32) {
        use std::num::Float;

        match self.state {
            PlayerState::Stand(ref s) => {
                (Float::floor(s.x), Float::floor(s.y))
            },
            PlayerState::Digging(ref s) => {
                (Float::floor(s.x), Float::floor(s.y))
            },
            PlayerState::Emerging(ref s) => {
                (Float::floor(s.x), Float::floor(s.y))
            }
        }
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        let (x, y) = self.get_pos();
        (x, y, x + 16.0, y + 16.0)
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

    pub fn add_drill(&mut self) {
        self.item = PlayerItem::Drill(PlayerItemDrill {
            phase: 0.0
        });
    }
}
