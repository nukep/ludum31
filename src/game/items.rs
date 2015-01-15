use super::level::{Level, Tiles};
use super::wrapping::Screen;
use super::collision;
use super::rect::{Point, Rect};

pub struct Poof {
    pub xy: Point<f32>,
    pub phase: f32
}

#[derive(Clone, Copy)]
pub struct Bullet {
    pub xy: Point<f32>,
    pub vel_x: f32,
    pub phase: f32,
    timeout: uint
}

impl Bullet {
    pub fn get_rect(&self) -> Rect<f32> {
        Rect::new(self.xy, (16.0, 1.0))
    }
}

pub struct Useless {
    pub xy: Point<f32>,
    pub phase: f32
}

pub struct Switch {
    pub trigger: u8,
    pub triggered_by: Option<u8>,
    pub xy: Point<f32>,
    pub is_down: bool,
    pub visible: bool,
    release_timeout: u8
}

impl Switch {
    pub fn get_rect(&self) -> Rect<f32> {
        Rect::new(self.xy, (16.0, 16.0))
    }
}

pub struct Chest {
    pub triggered_by: Option<u8>,
    pub trigger: Option<u8>,
    pub explode_trigger: Option<u8>,
    pub xy: Point<f32>,
    pub visible: bool,
    pub phase: f32,
    pub is_static: bool,
    opened: bool,
    original_xy: Point<f32>,
    fall_distance: f32,
    fall_phase: f32,
    contains: ChestItem
}

impl Chest {
    pub fn spawn(&mut self) {
        self.xy = self.original_xy;
        self.visible = true;
        self.fall_phase = 0.0;
    }

    pub fn get_rect(&self) -> Rect<f32> {
        Rect::new(self.xy, (16.0, 16.0))
    }
}

#[derive(Copy)]
pub enum ChestItem {
    UselessPoints,
    Drill,
    Gun,
    None
}

pub struct Monster1 {
    pub xy: Point<f32>,
    pub visible: bool,
    pub phase: f32,
    triggered_by: Option<u8>,
}

impl Monster1 {
    pub fn spawn(&mut self) {
        self.visible = true;
    }

    pub fn get_rect(&self) -> Rect<f32> {
        Rect::new(self.xy, (16.0, 16.0))
    }
}

pub struct Monster2 {
    pub original_xy: Point<f32>,
    pub xy: Point<f32>,
    pub visible: bool,
    pub phase: f32,
    move_phase: f32,
    triggered_by: Option<u8>,
}

impl Monster2 {
    pub fn spawn(&mut self) {
        self.visible = true;
        self.xy = self.original_xy;
    }

    pub fn get_rect(&self) -> Rect<f32> {
        Rect::new(self.xy, (16.0, 16.0))
    }
}

pub struct Beanstalk {
    pub xy: Point<f32>,
    pub height: uint,
    pub visible: bool,
    triggered_by: Option<u8>,
}

impl Beanstalk {
    pub fn spawn(&mut self) {
        self.visible = true;
    }

    pub fn get_rect(&self) -> Rect<f32> {
        Rect::new(self.xy, (16.0, self.height as f32 * 16.0))
    }
}

pub struct Key {
    pub xy: Point<f32>,
    pub is_sticky: bool,
    pub visible: bool,
    vel_y: f32,
    to_y: f32
}

impl Key {
    pub fn is_free(&self) -> bool {
        !self.is_sticky && self.visible
    }

    pub fn step(&mut self, screen: &Screen) {
        let p = self.xy.offset(screen, 0.0, self.vel_y);
        self.xy = if p.xy().1 > self.to_y { p.set_y(screen, self.to_y) } else { p };

        self.vel_y += 0.1;
    }

    pub fn unstick(&mut self) {
        self.is_sticky = false;
    }

    pub fn get_rect(&self) -> Rect<f32> {
        Rect::new(self.xy, (16.0, 16.0))
    }
}

pub struct Message {
    pub xy: Point<f32>,
    pub width: uint,
    pub height: uint,
    pub tiles: Vec<u16>,
    pub triggered_by: Option<u8>,
    pub visible: bool,
}

pub struct DynamicItems {
    pub poofs: Vec<Poof>,
    pub bullets: Vec<Bullet>,
    pub useless: Vec<Useless>,
    pub switches: Vec<Switch>,
    pub chests: Vec<Chest>,
    pub monsters1: Vec<Monster1>,
    pub monsters2: Vec<Monster2>,
    pub beanstalks: Vec<Beanstalk>,
    pub keys: Vec<Key>,
    pub messages: Vec<Message>,

    screen: Screen
}

impl DynamicItems {
    pub fn new(level: &Level) -> DynamicItems {
        let screen = level.get_screen();

        let switches = level.switches.iter().map(|s| {
            Switch {
                trigger: s.trigger,
                triggered_by: s.triggered_by,
                visible: match s.triggered_by { Some(_) => false, None => true },
                xy: Point::new(&screen, (s.x, s.y)),
                is_down: false,
                release_timeout: 0
            }
        }).collect();

        let chests = level.chests.iter().map(|s| {
            Chest {
                triggered_by: s.triggered_by,
                trigger: s.trigger,
                explode_trigger: s.explode_trigger,
                xy: Point::new(&screen, (s.x, s.y)),
                visible: match s.triggered_by { Some(_) => false, None => true },
                phase: 0.0,
                is_static: s.is_static,
                original_xy: Point::new(&screen, (s.x, s.y)),
                fall_distance: s.fall_distance,
                fall_phase: 0.0,
                opened: false,
                contains: match s.contains.as_slice() {
                    "useless" => ChestItem::UselessPoints,
                    "drill" => ChestItem::Drill,
                    "gun" => ChestItem::Gun,
                    "none" => ChestItem::None,
                    e => panic!("Unknown item: {}", e)
                }
            }
        }).collect();

        let monsters1 = level.monsters1.iter().map(|s| {
            Monster1 {
                xy: Point::new(&screen, (s.x, s.y)),
                visible: false,
                triggered_by: s.triggered_by,
                phase: 0.0
            }
        }).collect();

        let monsters2 = level.monsters2.iter().map(|s| {
            Monster2 {
                original_xy: Point::new(&screen, (s.x, s.y)),
                xy: Point::new(&screen, (s.x, s.y)),
                visible: false,
                triggered_by: s.triggered_by,
                phase: 0.0,
                move_phase: 0.0
            }
        }).collect();

        let beanstalks = level.beanstalks.iter().map(|s| {
            Beanstalk {
                xy: Point::new(&screen, (s.x, s.y)),
                height: s.height,
                visible: false,
                triggered_by: s.triggered_by
            }
        }).collect();

        let keys = level.sticky_keys.iter().map(|s| {
            Key {
                xy: Point::new(&screen, (s.x, s.y)),
                is_sticky: true,
                visible: true,
                vel_y: 0.0,
                to_y: s.y + s.fall_distance
            }
        }).collect();

        let messages = level.messages.iter().map(|s| {
            Message {
                xy: Point::new(&screen, (s.x, s.y)),
                width: s.width,
                height: s.height,
                tiles: s.tiles.clone(),
                triggered_by: s.triggered_by,
                visible: false,
            }
        }).collect();

        DynamicItems {
            poofs: Vec::new(),
            bullets: Vec::new(),
            useless: Vec::new(),
            switches: switches,
            chests: chests,
            monsters1: monsters1,
            monsters2: monsters2,
            beanstalks: beanstalks,
            keys: keys,
            messages: messages,
            screen: screen
        }
    }

    pub fn trigger(&mut self, id: u8) -> bool {
        let mut did_something = false;

        for switch in self.switches.iter_mut().filter(|c| c.trigger == id) {
            switch.is_down = true;
            switch.release_timeout = 60;
        }

        let mut poof_list: Vec<Point<f32>> = Vec::new();

        for switch in self.switches.iter_mut().filter(|c| c.triggered_by == Some(id) && !c.visible) {
            switch.visible = true;
            poof_list.push(switch.xy);
            did_something = true;
        }

        for chest in self.chests.iter_mut().filter(|c| c.triggered_by == Some(id) && !c.visible) {
            chest.spawn();
            poof_list.push(chest.xy);
            did_something = true;
        }

        for monster1 in self.monsters1.iter_mut().filter(|m| m.triggered_by == Some(id) && !m.visible) {
            monster1.spawn();
            poof_list.push(monster1.xy);
            did_something = true;
        }

        for monster2 in self.monsters2.iter_mut().filter(|m| m.triggered_by == Some(id) && !m.visible) {
            monster2.spawn();
            poof_list.push(monster2.xy);
            did_something = true;
        }

        for beanstalk in self.beanstalks.iter_mut().filter(|m| m.triggered_by == Some(id) && !m.visible) {
            beanstalk.spawn();
            for y in range(0u, beanstalk.height) {
                poof_list.push(beanstalk.xy.offset(&self.screen, 0.0, y as f32 * 16.0));
            }
            did_something = true;
        }

        for message in self.messages.iter_mut().filter(|m| m.triggered_by == Some(id) && !m.visible) {
            message.visible = true;
            did_something = true;
        }

        let screen = self.screen;

        for poof in poof_list.iter() {
            self.add_poof(poof.offset(&screen, -5.0, -5.0));
            self.add_poof(poof.offset(&screen, 5.0, 5.0));
            self.add_poof(poof.offset(&screen, 12.0, -3.0));
        }

        did_something
    }

    pub fn switch_hit_test(&self, rect: &Rect<f32>) -> Vec<&Switch> {
        // Switches love triggers

        self.switches.iter().filter(|s| s.visible).filter_map(|switch| {
            let hit = collision::test_rects(rect, &switch.get_rect());

            if hit { Some(switch) }
            else { None }
        }).collect()
    }

    pub fn try_open_chest(&mut self, rect: &Rect<f32>) -> Vec<(f32, f32, ChestItem)> {
        let mut opened_chest = false;
        let mut triggers: Vec<u8> = Vec::new();

        let items = self.chests.iter_mut().filter(|c| c.visible && !c.opened).filter_map(|chest| {
            let hit = collision::test_rects(rect, &chest.get_rect());
            if hit {
                opened_chest = true;
                chest.opened = true;
                match chest.trigger {
                    Some(trigger) => triggers.push(trigger),
                    None => ()
                };
                let (x, y) = chest.xy.xy();
                Some((x, y, chest.contains))
            }
            else { None }
        }).collect();

        for trigger in triggers.iter() {
            self.trigger(*trigger);
        }

        items
    }

    pub fn try_take_keys(&mut self, rect: &Rect<f32>) -> uint {
        let mut count = 0u;
        for key in self.keys.iter_mut().filter(|k| k.is_free()) {
            if collision::test_rects(rect, &key.get_rect()) {
                key.visible = false;
                count += 1;
            }
        }
        count
    }

    pub fn beanstalk_exists(&mut self, rect: &Rect<f32>) -> Option<Rect<f32>> {
        for beanstalk in self.beanstalks.iter().filter(|b| b.visible) {
            if collision::test_rects(rect, &beanstalk.get_rect()) {
                return Some(beanstalk.get_rect());
            }
        }
        None
    }

    pub fn add_poof(&mut self, xy: Point<f32>) {
        self.poofs.push(Poof {
            xy: xy,
            phase: 0.0
        });
    }

    pub fn add_bullet(&mut self, xy: Point<f32>, vel_x: f32) {
        self.bullets.push(Bullet {
            xy: xy,
            vel_x: vel_x,
            phase: 0.0,
            timeout: 40
        });
    }

    pub fn add_useless_points(&mut self, xy: Point<f32>) {
        self.useless.push(Useless {
            xy: xy,
            phase: 0.0
        });
    }

    fn step_poofs(&mut self) {
        let new_poofs = self.poofs.iter().filter_map(|poof| {
            let phase = poof.phase + 0.05;
            if phase >= 1.0 {
                None
            } else {
                Some(Poof {
                    xy: poof.xy,
                    phase: phase
                })
            }
        }).collect();

        self.poofs = new_poofs;
    }

    fn step_bullets(&mut self, screen: &Screen) {
        let new_bullets = self.bullets.iter().filter_map(|bullet| {
            let mut phase = bullet.phase + 0.3;
            if phase >= 1.0 { phase = 1.0; }

            let new_xy = bullet.xy.offset(screen, bullet.vel_x, 0.0);

            if bullet.timeout - 1 == 0 {
                None
            } else {
                Some(Bullet {
                    xy: new_xy,
                    vel_x: bullet.vel_x,
                    phase: phase,
                    timeout: bullet.timeout - 1
                })
            }
        }).collect();

        self.bullets = new_bullets;
    }

    fn step_useless(&mut self) {
        let new_useless = self.useless.iter().filter_map(|useless| {
            let phase = useless.phase + 0.03;
            if phase >= 1.0 {
                None
            } else {
                Some(Useless {
                    xy: useless.xy.offset(&self.screen, 0.0, -0.5),
                    phase: phase
                })
            }
        }).collect();

        self.useless = new_useless;
    }

    pub fn bullet_item_collision(&mut self, tiles: &Tiles) {
        // Annihilate both the bullet and the item on collision

        let mut poof_list: Vec<Point<f32>> = Vec::new();

        {
        let screen = self.screen;
        let monsters1 = &mut self.monsters1;
        let monsters2 = &mut self.monsters2;
        let chests = &mut self.chests;
        let keys = &mut self.keys;

        let new_bullets = self.bullets.iter().filter_map(|bullet| {
            let rect = bullet.get_rect();
            let mut bullet_alive = true;

            for monster1 in monsters1.iter_mut().filter(|m| m.visible) {
                if collision::test_rects(&rect, &monster1.get_rect()) {
                    poof_list.push(monster1.xy);
                    monster1.visible = false;
                    bullet_alive = false;
                }
            }

            for monster2 in monsters2.iter_mut().filter(|m| m.visible) {
                if collision::test_rects(&rect, &monster2.get_rect()) {
                    poof_list.push(monster2.xy);
                    monster2.visible = false;
                    bullet_alive = false;
                }
            }

            for chest in chests.iter_mut().filter(|c| c.visible) {
                if collision::test_rects(&rect, &chest.get_rect()) {
                    poof_list.push(chest.xy);
                    chest.visible = false;
                    bullet_alive = false;
                }
            }

            for key in keys.iter_mut().filter(|k| k.is_sticky) {
                if collision::test_rects(&rect, &key.get_rect()) {
                    key.unstick();
                    bullet_alive = false;
                }
            }

            if let Some(_) = tiles.collision_tile(&rect, (None, None)) {
                poof_list.push(bullet.xy.offset(&screen, -8.0, -8.0));
                bullet_alive = false;
            }

            if bullet_alive { Some(bullet.clone()) }
            else { None }
        }).collect();

        self.bullets = new_bullets;
        }

        for poof in poof_list.iter() {
            self.add_poof(*poof);
        }
    }

    /// Returns (true, _) if items have been moved.
    /// Returns (_, true) if items have been destroyed.
    pub fn adjust_to_scroll_boundary(&mut self, screen: &Screen, tiles: &Tiles, x_line: f32, x_inc: bool, x_dec: bool) -> (bool, bool) {
        let width = screen.width;

        let do_collision = |&: rect: &Rect<f32>| -> (Rect<f32>, bool, bool) {
            let mut moved = false;

            let new_rect = if x_inc {
                if collision::test_rect_vert_line(rect, x_line, width) {
                    moved = true;
                    rect.set_x(screen, x_line)
                } else { *rect }
            } else if x_dec {
                if collision::test_rect_vert_line(rect, x_line, width) {
                    moved = true;
                    rect.set_x(screen, (x_line - rect.width() + width) % width)
                } else { *rect}
            } else { *rect };

            let destroy = if let Some((_, _)) = tiles.collision_tile(&new_rect, (None, None)) { true }
            else { false };

            (new_rect, moved, destroy)
        };

        // Item sliding and crushing occurs here
        let mut moved = false;
        let mut destroyed = false;

        let mut poof_list: Vec<Point<f32>> = Vec::new();
        let mut triggers: Vec<u8> = Vec::new();

        for chest in self.chests.iter_mut().filter(|c| c.visible && !c.is_static ) {
            let (new_rect, mov, destroy) = do_collision(&chest.get_rect());

            chest.xy = new_rect.left_top();

            if mov { moved = true }

            if destroy {
                chest.visible = false;
                if let Some(trigger) = chest.explode_trigger {
                    triggers.push(trigger);
                }
                poof_list.push(chest.xy);
                destroyed = true;
            }
        }

        for monster1 in self.monsters1.iter_mut().filter(|m| m.visible) {
            let (new_rect, mov, destroy) = do_collision(&monster1.get_rect());

            monster1.xy = new_rect.left_top();

            if mov { moved = true }

            if destroy {
                monster1.visible = false;
                poof_list.push(monster1.xy);
                destroyed = true;
            }
        }

        for poof in poof_list.iter() {
            self.add_poof(*poof);
        }

        for trigger in triggers.iter() {
            self.trigger(*trigger);
        }

        (moved, destroyed)
    }

    fn step_chests(&mut self) {
        fn lerp(a: f32, b: f32, p: f32) -> f32 { (b-a)*p + a }
        fn curve(x: f32) -> f32 {
            use std::num::Float;
            use std::f32::consts::FRAC_PI_2;

            let coeff = 1.4;

            1.0 - Float::sin(((x*coeff)-coeff)*FRAC_PI_2) / Float::sin((-coeff)*FRAC_PI_2)
        }

        for chest in self.chests.iter_mut().filter(|c| c.visible && c.fall_phase < 1.0) {
            let fall_rate = if chest.fall_distance == 0.0 { 1.0 } else { 16.0 / chest.fall_distance };

            chest.fall_phase += 0.06 * fall_rate;
            if chest.fall_phase > 1.0 { chest.fall_phase = 1.0 }
            let y = lerp(chest.original_xy.y(), chest.original_xy.y() + chest.fall_distance, curve(chest.fall_phase));
            chest.xy = chest.xy.set_y(&self.screen, y);
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

            let x = monster2.original_xy.x();
            let new_x = lerp(x - 16.0, x + 16.0, p);
            monster2.xy = monster2.xy.set_x(&self.screen, new_x);
        }
    }

    fn step_keys(&mut self) {
        for key in self.keys.iter_mut().filter(|k| k.is_free()) {
            key.step(&self.screen)
        }
    }

    pub fn rect_hits_monsters(&self, rect: &Rect<f32>) -> bool {
        for monster1 in self.monsters1.iter().filter(|m| m.visible) {
            if collision::test_rects(rect, &monster1.get_rect()) {
                return true;
            }
        }
        for monster2 in self.monsters2.iter().filter(|m| m.visible) {
            if collision::test_rects(rect, &monster2.get_rect()) {
                return true;
            }
        }
        false
    }

    pub fn step(&mut self, screen: &Screen) {
        self.step_poofs();
        self.step_bullets(screen);
        self.step_useless();
        self.step_chests();
        self.step_monsters();
        self.step_keys();
    }
}
