use super::level::Level;
use super::collision;
use super::rect::RectExt;

pub struct Poof {
    pub x: f32,
    pub y: f32,
    pub phase: f32
}

pub struct Switch {
    pub trigger: u8,
    pub x: f32,
    pub y: f32,
    pub is_down: bool,
    release_timeout: u8
}

impl Switch {
    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.x + 16.0, self.y + 16.0)
    }
}

pub struct Chest {
    pub triggered_by: Option<u8>,
    pub trigger: Option<u8>,
    pub x: f32,
    pub y: f32,
    pub visible: bool,
    pub phase: f32,
    pub is_static: bool,
    opened: bool,
    original_x: f32,
    original_y: f32,
    fall_distance: f32,
    fall_phase: f32,
    contains: ChestItem
}

impl Chest {
    pub fn spawn(&mut self) {
        self.x = self.original_x;
        self.y = self.original_y;
        self.visible = true;
        self.fall_phase = 0.0;
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.x + 16.0, self.y + 16.0)
    }
}

pub enum ChestItem {
    UselessPoints,
    Drill,
    Gun
}

pub struct DynamicItems {
    pub poofs: Vec<Poof>,
    pub switches: Vec<Switch>,
    pub chests: Vec<Chest>
}

impl DynamicItems {
    pub fn new(level: &Level) -> DynamicItems {
        let switches = level.switches.iter().map(|s| {
            Switch {
                trigger: s.trigger,
                x: s.x,
                y: s.y,
                is_down: false,
                release_timeout: 0
            }
        }).collect();

        let chests = level.chests.iter().map(|s| {
            Chest {
                triggered_by: s.triggered_by,
                trigger: s.trigger,
                x: s.x,
                y: s.y,
                visible: match s.triggered_by { Some(_) => false, None => true },
                phase: 0.0,
                is_static: s.is_static,
                original_x: s.x,
                original_y: s.y,
                fall_distance: s.fall_distance,
                fall_phase: 0.0,
                opened: false,
                contains: match s.contains.as_slice() {
                    "useless" => ChestItem::UselessPoints,
                    "drill" => ChestItem::Drill,
                    "gun" => ChestItem::Gun,
                    e => panic!("Unknown item: {}", e)
                }
            }
        }).collect();

        DynamicItems {
            poofs: Vec::new(),
            switches: switches,
            chests: chests
        }
    }

    pub fn trigger(&mut self, id: u8) -> bool {
        let mut did_something = false;

        for switch in self.switches.iter_mut().filter(|c| c.trigger == id) {
            switch.is_down = true;
            switch.release_timeout = 60;
        }

        let mut poof_list: Vec<(f32, f32)> = Vec::new();

        for chest in self.chests.iter_mut().filter(|c| c.triggered_by == Some(id) && !c.visible) {
            chest.spawn();
            poof_list.push((chest.x-5.0, chest.y-5.0));
            poof_list.push((chest.x+5.0, chest.y+5.0));
            poof_list.push((chest.x+12.0, chest.y-3.0));
            did_something = true;
        }

        for poof in poof_list.iter() {
            let &(x, y) = poof;
            self.add_poof(x, y);
        }

        did_something
    }

    pub fn switch_hit_test(&self, x: f32, y: f32, w: f32, h: f32) -> Vec<&Switch> {
        // Switches love triggers

        self.switches.iter().filter_map(|switch| {
            let hit = collision::test_rects((x, y, x+w, y+h), switch.get_rect());

            if hit { Some(switch) }
            else { None }
        }).collect()
    }

    pub fn try_open_chest(&mut self, rect: (f32, f32, f32, f32)) -> Vec<ChestItem> {
        let mut opened_chest = false;
        self.chests.iter_mut().filter(|c| c.visible && !c.opened).filter_map(|chest| {
            let hit = collision::test_rects(rect, chest.get_rect());
            if hit {
                opened_chest = true;
                chest.opened = true;
                Some(chest.contains)
            }
            else { None }
        }).collect()
    }

    pub fn add_poof(&mut self, x: f32, y: f32) {
        self.poofs.push(Poof {
            x: x,
            y: y,
            phase: 0.0
        });
    }

    pub fn step_poofs(&mut self) {
        let new_poofs = self.poofs.iter().filter_map(|poof| {
            let phase = poof.phase + 0.05;
            if phase >= 1.0 {
                None
            } else {
                Some(Poof {
                    x: poof.x,
                    y: poof.y,
                    phase: phase
                })
            }
        }).collect();

        self.poofs = new_poofs;
    }

    /// Returns (true, _) if items have been moved.
    /// Returns (_, true) if items have been destroyed.
    pub fn adjust_to_scroll_boundary(&mut self, level: &Level, x_line: f32, y_line: f32, x_inc: bool, y_inc: bool, x_dec: bool, y_dec: bool) -> (bool, bool) {
        // Item sliding and crushing occurs here
        let mut moved = false;
        let mut destroyed = false;

        let mut poof_list: Vec<(f32, f32)> = Vec::new();

        let (width, height) = level.level_size_as_f32();

        for chest in self.chests.iter_mut().filter(|c| c.visible && !c.is_static ) {
            let rect = chest.get_rect();
            if x_inc {
                if collision::test_rect_vert_line(rect, x_line, width) {
                    chest.x = x_line;
                    moved = true;
                }
            } else if x_dec {
                if collision::test_rect_vert_line(rect, x_line, width) {
                    chest.x = (x_line - 16.0 + width) % width;
                    moved = true;
                }
            }
            let new_rect = chest.get_rect();
            let destroy = if let Some((_, _)) = level.collision_tile(new_rect, (None, None)) { true }
                else { false };

            if destroy {
                chest.visible = false;
                poof_list.push((chest.x, chest.y));
                destroyed = true;
            }
        }

        for poof in poof_list.iter() {
            let &(x, y) = poof;
            self.add_poof(x, y);
        }

        (moved, destroyed)
    }

    pub fn step_chests(&mut self) {
        fn lerp(a: f32, b: f32, p: f32) -> f32 { (b-a)*p + a }
        fn curve(x: f32) -> f32 {
            use std::num::{Float, FloatMath};

            let coeff = 1.4;

            1.0 - FloatMath::sin(((x*coeff)-coeff)*Float::frac_pi_2()) / FloatMath::sin((-coeff)*Float::frac_pi_2())
        }

        for chest in self.chests.iter_mut().filter(|c| c.visible && c.fall_phase < 1.0) {
            chest.fall_phase += 0.03;
            if chest.fall_phase > 1.0 { chest.fall_phase = 1.0 }
            chest.y = lerp(chest.original_y, chest.original_y + chest.fall_distance, curve(chest.fall_phase));
        }

        for chest in self.chests.iter_mut().filter(|c| c.visible && c.opened && c.phase < 1.0) {
            chest.phase += 0.03;
            if chest.phase > 1.0 { chest.phase = 1.0 }
        }
    }
}
