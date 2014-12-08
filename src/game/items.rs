use super::level::Level;
use super::collision;
use super::rect::RectExt;

pub struct Poof {
    pub x: f32,
    pub y: f32,
    pub phase: f32
}

#[deriving(Clone, Copy)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub phase: f32,
    timeout: uint
}

impl Bullet {
    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.x + 16.0, self.y + 1.0)
    }
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

pub struct Monster1 {
    pub x: f32,
    pub y: f32,
    pub visible: bool,
    pub phase: f32,
    triggered_by: Option<u8>,
}

impl Monster1 {
    pub fn spawn(&mut self) {
        self.visible = true;
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.x + 16.0, self.y + 16.0)
    }
}

pub struct Monster2 {
    pub original_x: f32,
    pub original_y: f32,
    pub x: f32,
    pub y: f32,
    pub visible: bool,
    pub phase: f32,
    move_phase: f32,
    triggered_by: Option<u8>,
}

impl Monster2 {
    pub fn spawn(&mut self) {
        self.visible = true;
        self.x = self.original_x;
        self.y = self.original_y;
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.x + 16.0, self.y + 16.0)
    }
}

pub struct Beanstalk {
    pub x: f32,
    pub y: f32,
    pub height: uint,
    pub visible: bool,
    triggered_by: Option<u8>,
}

impl Beanstalk {
    pub fn spawn(&mut self) {
        self.visible = true;
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.x + 16.0, self.y + self.height as f32 * 16.0)
    }
}

pub struct Key {
    pub x: f32,
    pub y: f32,
    pub is_sticky: bool,
    pub visible: bool
}

impl Key {
    pub fn unstick(&mut self) {
        self.is_sticky = false;
    }

    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.x + 16.0, self.y + 16.0)
    }
}

pub struct DynamicItems {
    pub poofs: Vec<Poof>,
    pub bullets: Vec<Bullet>,
    pub switches: Vec<Switch>,
    pub chests: Vec<Chest>,
    pub monsters1: Vec<Monster1>,
    pub monsters2: Vec<Monster2>,
    pub beanstalks: Vec<Beanstalk>,
    pub keys: Vec<Key>,
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

        let monsters1 = level.monsters1.iter().map(|s| {
            Monster1 {
                x: s.x,
                y: s.y,
                visible: false,
                triggered_by: s.triggered_by,
                phase: 0.0
            }
        }).collect();

        let monsters2 = level.monsters2.iter().map(|s| {
            Monster2 {
                original_x: s.x,
                original_y: s.y,
                x: s.x,
                y: s.y,
                visible: false,
                triggered_by: s.triggered_by,
                phase: 0.0,
                move_phase: 0.0
            }
        }).collect();

        let beanstalks = level.beanstalks.iter().map(|s| {
            Beanstalk {
                x: s.x,
                y: s.y,
                height: s.height,
                visible: false,
                triggered_by: s.triggered_by
            }
        }).collect();

        let keys = level.sticky_keys.iter().map(|s| {
            Key {
                x: s.x,
                y: s.y,
                is_sticky: true,
                visible: true
            }
        }).collect();

        DynamicItems {
            poofs: Vec::new(),
            bullets: Vec::new(),
            switches: switches,
            chests: chests,
            monsters1: monsters1,
            monsters2: monsters2,
            beanstalks: beanstalks,
            keys: keys,
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
            poof_list.push((chest.x, chest.y));
            did_something = true;
        }

        for monster1 in self.monsters1.iter_mut().filter(|m| m.triggered_by == Some(id) && !m.visible) {
            monster1.spawn();
            poof_list.push((monster1.x, monster1.y));
            did_something = true;
        }

        for monster2 in self.monsters2.iter_mut().filter(|m| m.triggered_by == Some(id) && !m.visible) {
            monster2.spawn();
            poof_list.push((monster2.x, monster2.y));
            did_something = true;
        }

        for beanstalk in self.beanstalks.iter_mut().filter(|m| m.triggered_by == Some(id) && !m.visible) {
            beanstalk.spawn();
            for y in range(0u, beanstalk.height) {
                poof_list.push((beanstalk.x, beanstalk.y + y as f32 * 16.0));
            }
            did_something = true;
        }

        for poof in poof_list.iter() {
            let &(x, y) = poof;
            self.add_poof(x-5.0, y-5.0);
            self.add_poof(x+5.0, y+5.0);
            self.add_poof(x+12.0, y-3.0);
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
        let mut triggers: Vec<u8> = Vec::new();

        let items = self.chests.iter_mut().filter(|c| c.visible && !c.opened).filter_map(|chest| {
            let hit = collision::test_rects(rect, chest.get_rect());
            if hit {
                opened_chest = true;
                chest.opened = true;
                match chest.trigger {
                    Some(trigger) => triggers.push(trigger),
                    None => ()
                };
                Some(chest.contains)
            }
            else { None }
        }).collect();

        for trigger in triggers.iter() {
            self.trigger(*trigger);
        }

        items
    }

    pub fn beanstalk_exists(&mut self, rect: (f32, f32, f32, f32)) -> Option<(f32, f32, f32, f32)> {
        for beanstalk in self.beanstalks.iter().filter(|b| b.visible) {
            if collision::test_rects(rect, beanstalk.get_rect()) {
                return Some(beanstalk.get_rect());
            }
        }
        None
    }

    pub fn add_poof(&mut self, x: f32, y: f32) {
        self.poofs.push(Poof {
            x: x,
            y: y,
            phase: 0.0
        });
    }

    pub fn add_bullet(&mut self, x: f32, y: f32, vel_x: f32) {
        self.bullets.push(Bullet {
            x: x,
            y: y,
            vel_x: vel_x,
            phase: 0.0,
            timeout: 40
        });
    }

    fn step_poofs(&mut self) {
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

    fn step_bullets(&mut self, level: &Level) {
        let new_bullets = self.bullets.iter().filter_map(|bullet| {
            let mut phase = bullet.phase + 0.3;
            if phase >= 1.0 { phase = 1.0; }

            let new_coord = level.wrap_coordinates((bullet.x + bullet.vel_x, bullet.y));
            let x = new_coord.val0();

            if bullet.timeout - 1 == 0 {
                None
            } else {
                Some(Bullet {
                    x: x,
                    y: bullet.y,
                    vel_x: bullet.vel_x,
                    phase: phase,
                    timeout: bullet.timeout - 1
                })
            }
        }).collect();

        self.bullets = new_bullets;
    }

    pub fn bullet_item_collision(&mut self, level: &Level) {
        // Annihilate both the bullet and the item on collision

        let mut poof_list: Vec<(f32, f32)> = Vec::new();

        {
        let monsters1 = &mut self.monsters1;
        let monsters2 = &mut self.monsters2;
        let chests = &mut self.chests;
        let keys = &mut self.keys;

        let new_bullets = self.bullets.iter().filter_map(|bullet| {
            let rect = bullet.get_rect();
            let mut bullet_alive = true;

            for monster1 in monsters1.iter_mut().filter(|m| m.visible) {
                if collision::test_rects(rect, monster1.get_rect()) {
                    poof_list.push((monster1.x, monster1.y));
                    monster1.visible = false;
                    bullet_alive = false;
                }
            }

            for monster2 in monsters2.iter_mut().filter(|m| m.visible) {
                if collision::test_rects(rect, monster2.get_rect()) {
                    poof_list.push((monster2.x, monster2.y));
                    monster2.visible = false;
                    bullet_alive = false;
                }
            }

            for chest in chests.iter_mut().filter(|c| c.visible) {
                if collision::test_rects(rect, chest.get_rect()) {
                    poof_list.push((chest.x, chest.y));
                    chest.visible = false;
                    bullet_alive = false;
                }
            }

            for key in keys.iter_mut().filter(|k| k.is_sticky) {
                if collision::test_rects(rect, key.get_rect()) {
                    key.unstick();
                    bullet_alive = false;
                }
            }

            if let Some(_) = level.collision_tile(rect, (None, None)) {
                poof_list.push((bullet.x - 8.0, bullet.y - 8.0));
                bullet_alive = false;
            }

            if bullet_alive { Some(bullet.clone()) }
            else { None }
        }).collect();

        self.bullets = new_bullets;
        }

        for poof in poof_list.iter() {
            let &(x, y) = poof;
            self.add_poof(x, y);
        }
    }

    /// Returns (true, _) if items have been moved.
    /// Returns (_, true) if items have been destroyed.
    pub fn adjust_to_scroll_boundary(&mut self, level: &Level, x_line: f32, x_inc: bool, x_dec: bool) -> (bool, bool) {
        let (width, _) = level.level_size_as_f32();

        let do_collision = |rect: (f32, f32, f32, f32)| -> ((f32, f32, f32, f32), bool, bool) {
            let mut moved = false;

            let new_rect = if x_inc {
                if collision::test_rect_vert_line(rect, x_line, width) {
                    moved = true;
                    rect.set_x(level, x_line)
                } else { rect }
            } else if x_dec {
                if collision::test_rect_vert_line(rect, x_line, width) {
                    moved = true;
                    rect.set_x(level, (x_line - rect.width() + width) % width)
                } else { rect}
            } else { rect };

            let destroy = if let Some((_, _)) = level.collision_tile(new_rect, (None, None)) { true }
            else { false };

            (new_rect, moved, destroy)
        };

        // Item sliding and crushing occurs here
        let mut moved = false;
        let mut destroyed = false;

        let mut poof_list: Vec<(f32, f32)> = Vec::new();

        for chest in self.chests.iter_mut().filter(|c| c.visible && !c.is_static ) {
            let (new_rect, mov, destroy) = do_collision(chest.get_rect());

            chest.x = new_rect.x();
            chest.y = new_rect.y();

            if mov { moved = true }

            if destroy {
                chest.visible = false;
                poof_list.push((chest.x, chest.y));
                destroyed = true;
            }
        }

        for monster1 in self.monsters1.iter_mut().filter(|m| m.visible) {
            let (new_rect, mov, destroy) = do_collision(monster1.get_rect());

            monster1.x = new_rect.x();
            monster1.y = new_rect.y();

            if mov { moved = true }

            if destroy {
                monster1.visible = false;
                poof_list.push((monster1.x, monster1.y));
                destroyed = true;
            }
        }

        for poof in poof_list.iter() {
            let &(x, y) = poof;
            self.add_poof(x, y);
        }

        (moved, destroyed)
    }

    fn step_chests(&mut self) {
        fn lerp(a: f32, b: f32, p: f32) -> f32 { (b-a)*p + a }
        fn curve(x: f32) -> f32 {
            use std::num::FloatMath;
            use std::f32::consts::FRAC_PI_2;

            let coeff = 1.4;

            1.0 - FloatMath::sin(((x*coeff)-coeff)*FRAC_PI_2) / FloatMath::sin((-coeff)*FRAC_PI_2)
        }

        for chest in self.chests.iter_mut().filter(|c| c.visible && c.fall_phase < 1.0) {
            let fall_rate = if chest.fall_distance == 0.0 { 1.0 } else { 16.0 / chest.fall_distance };

            chest.fall_phase += 0.06 * fall_rate;
            if chest.fall_phase > 1.0 { chest.fall_phase = 1.0 }
            chest.y = lerp(chest.original_y, chest.original_y + chest.fall_distance, curve(chest.fall_phase));
        }

        for chest in self.chests.iter_mut().filter(|c| c.visible && c.opened && c.phase < 1.0) {
            chest.phase += 0.03;
            if chest.phase > 1.0 { chest.phase = 1.0 }
        }
    }

    fn step_monsters(&mut self) {
        for monster1 in self.monsters1.iter_mut().filter(|m| m.visible) {
            monster1.phase = (monster1.phase + 0.015) % 1.0;
        }
        for monster2 in self.monsters2.iter_mut().filter(|m| m.visible) {
            fn lerp(a: f32, b: f32, p: f32) -> f32 { (b-a)*p + a }

            monster2.phase = (monster2.phase + 0.015) % 1.0;
            monster2.move_phase = (monster2.move_phase + 0.005) % 1.0;

            let p = match monster2.move_phase*2.0 {
                e @ 0.0...1.0 => e,
                e @ 1.0...2.0 => 1.0-(e-1.0),
                _ => 0.0
            };

            monster2.x = lerp(monster2.original_x - 16.0, monster2.original_x + 16.0, p);
        }
    }

    pub fn rect_hits_monsters(&self, rect: (f32, f32, f32, f32)) -> bool {
        for monster1 in self.monsters1.iter().filter(|m| m.visible) {
            if collision::test_rects(rect, monster1.get_rect()) {
                return true;
            }
        }
        for monster2 in self.monsters2.iter().filter(|m| m.visible) {
            if collision::test_rects(rect, monster2.get_rect()) {
                return true;
            }
        }
        false
    }

    pub fn step(&mut self, level: &Level) {
        self.step_poofs();
        self.step_bullets(level);
        self.step_chests();
        self.step_monsters();
    }
}
