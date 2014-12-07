use super::level::Level;
use super::collision;

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

pub struct Chest {
    pub triggered_by: Option<u8>,
    pub trigger: Option<u8>,
    pub x: f32,
    pub y: f32,
    pub visible: bool,
    pub phase: f32,
    pub is_static: bool,
    original_x: f32,
    original_y: f32,
    fall_distance: f32,
    fall_phase: f32
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
                fall_phase: 0.0
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

        for switch in self.switches.iter_mut().take_while(|c| c.trigger == id) {
            switch.is_down = true;
            switch.release_timeout = 60;
        }

        let mut poof_list: Vec<(f32, f32)> = Vec::new();

        for chest in self.chests.iter_mut().take_while(|c| c.triggered_by == Some(id) && !c.visible) {
            chest.visible = true;
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
            let hit = collision::test_rects((x, y, x+w, y+h), (switch.x, switch.y, switch.x+16.0, switch.y+16.0));

            if hit { Some(switch) }
            else { None }
        }).collect()
    }

    pub fn try_open_chest(&mut self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let hit_chests = self.chests.iter().filter_map(|chest| {
            let hit = collision::test_rects((x, y, x+w, y+h), (chest.x, chest.y, chest.x+16.0, chest.y+16.0));
            if hit { Some(chest) }
            else { None }
        });
        false
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

    pub fn step_falling_chests(&mut self) {
        fn lerp(a: f32, b: f32, p: f32) -> f32 { (b-a)*p + a }

        for chest in self.chests.iter_mut().take_while(|c| c.visible && c.fall_phase < 1.0) {
            chest.fall_phase += 0.05;
            chest.y = lerp(chest.original_y, chest.original_y + chest.fall_distance, chest.fall_phase);
        }
    }
}
