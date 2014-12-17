use std::num::FromStrRadix;
use serialize;
use super::rect::Rect;
use super::wrapping::Screen;

#[deriving(Clone)]
pub struct Tile {
    pub tile_type: TileType,
    pub flip_x: bool,
    pub flip_y: bool
}

impl Tile {
    pub fn empty() -> Tile {
        Tile {
            tile_type: TileType::from_id(0),
            flip_x: false,
            flip_y: false
        }
    }
}

#[deriving(Clone)]
pub struct TileType {
    pub id: u16,
    pub is_blocking: bool,
    pub can_dig: bool
}

impl TileType {
    pub fn from_id(id: u16) -> TileType {
        let is_blocking = match id {
            0 => false,
            // Dirt
            0x17 => false,
            0x23 => false,
            0x3A => false,
            // Exit
            0x2C => false,
            0x2D => false,
            // Coin
            0x21 => false,
            _ => true
        };
        let can_dig = match id {
            // Dirt
            0x16 => true,
            0x17 => true,
            0x23 => true,
            0x3A => true,
            _ => false
        };

        TileType {
            id: id,
            is_blocking: is_blocking,
            can_dig: can_dig
        }
    }
}

pub struct Switch {
    pub x: f32,
    pub y: f32,
    pub trigger: u8,
    pub triggered_by: Option<u8>,
}

pub struct Chest {
    pub x: f32,
    pub y: f32,
    pub trigger: Option<u8>,
    pub explode_trigger: Option<u8>,
    pub triggered_by: Option<u8>,
    pub poof: bool,
    pub is_static: bool,
    pub fall_distance: f32,
    pub contains: String
}

pub struct Beanstalk {
    pub x: f32,
    pub y: f32,
    pub height: uint,
    pub triggered_by: Option<u8>,
    pub poof: bool,
}

pub struct Monster1 {
    pub x: f32,
    pub y: f32,
    pub triggered_by: Option<u8>
}

pub struct Monster2 {
    pub x: f32,
    pub y: f32,
    pub triggered_by: Option<u8>
}

pub struct StickyKey {
    pub x: f32,
    pub y: f32,
    pub fall_distance: f32
}

pub struct Message {
    pub x: f32,
    pub y: f32,
    pub width: uint,
    pub height: uint,
    pub tiles: Vec<u16>,
    pub triggered_by: Option<u8>
}

pub struct SetTo {
    pub x: uint,
    pub y: uint,
    pub width: uint,
    pub height: uint,
    pub tile: Tile,
    pub triggered_by: Option<u8>
}

pub struct Tiles {
    width: u8,
    height: u8,
    screen: Screen,
    tiles: Vec<Tile>,
    tile_size: f32
}

impl Tiles {
    pub fn new(width: u8, height: u8, tiles: Vec<Tile>, tile_size: f32) -> Tiles {
        Tiles {
            width: width,
            height: height,
            screen: Screen::new(width as f32 * tile_size, height as f32 * tile_size),
            tiles: tiles,
            tile_size: tile_size
        }
    }

    pub fn get_tile(&self, x: u8, y: u8) -> &Tile {
        let offset = y as uint * self.width as uint + x as uint;
        self.tiles.index(&offset)
    }

    pub fn set_tile(&mut self, x: u8, y: u8, tile: Tile) {
        let offset = y as uint * self.width as uint + x as uint;
        *self.tiles.index_mut(&offset) = tile;
    }

    pub fn apply_set_to(&mut self, set_to: &SetTo) {
        for y in range(set_to.y, set_to.y + set_to.height) {
            for x in range(set_to.x, set_to.x + set_to.width) {
                self.set_tile(x as u8, y as u8, set_to.tile.clone());
            }
        }
    }

    pub fn iter(&self) -> LevelTileIterator {
        LevelTileIterator {
            tiles: &self.tiles,
            width: self.width as uint,
            index: 0
        }
    }

    fn nudge(tile_size: f32, x: f32, y: f32, left_top: (int, int), right_bottom: (int, int), direction: (Option<bool>, Option<bool>)) -> (f32, f32) {
        let (left, top) = left_top;
        let (right, bottom) = right_bottom;
        let (nudge_right, nudge_bottom) = direction;

        let new_left = match nudge_right {
            None => x,
            Some(false) => right as f32 * tile_size,
            Some(true) => left as f32 * tile_size
        };
        let new_top = match nudge_bottom {
            None => y,
            Some(false) => bottom as f32 * tile_size,
            Some(true) => top as f32 * tile_size
        };
        (new_left, new_top)
    }

    /// Direction: (right?, down?)
    /// Returns: new top_left
    pub fn collision_tile(&self, rect: &Rect<f32>, direction: (Option<bool>, Option<bool>)) -> Option<(f32, f32)> {
        let (x, y) = rect.left_top().xy();

        let (tiles, left_top, right_bottom) = self.get_tiles_in_rect(rect);

        let nudge = tiles.iter().any(|&(t, _, _)| t.tile_type.is_blocking);

        if nudge {
            Some(Tiles::nudge(self.tile_size, x, y, left_top, right_bottom, direction))
        } else {
            None
        }
    }

    pub fn collision_tile_digging(&self, rect: &Rect<f32>, direction: (Option<bool>, Option<bool>), get_emerge: bool) -> Option<(f32, f32, bool)> {
        let (x, y) = rect.left_top().xy();

        let (tiles, left_top, right_bottom) = self.get_tiles_in_rect(rect);

        let nudge = tiles.iter().any(|&(t, _, _)| !t.tile_type.can_dig);
        let emerge_hit: Vec<(int, int)> = tiles.iter().filter_map(|&(t, x, y)| {
            if t.tile_type.id == 0x16 { Some((x, y)) }
            else { None }
        }).collect();

        if get_emerge && emerge_hit.len() >= 1 {
            let (x, y) = emerge_hit[0];
            Some((x as f32 * self.tile_size, y as f32 * self.tile_size, true))
        } else {
            if nudge {
                let (x, y) = Tiles::nudge(self.tile_size, x, y, left_top, right_bottom, direction);
                Some((x, y, false))
            } else {
                None
            }
        }
    }

    fn get_left_top_tile_coord(&self, rect: &Rect<f32>) -> ((int, int), (int, int)) {
        use std::num::Float;

        let (x, y) = rect.left_top().xy();
        let (w, h) = rect.size();

        let (left, top) = {
            (Float::floor(x / self.tile_size) as int, Float::floor(y / self.tile_size) as int)
        };
        let (right, bottom) = {
            let (mut r, mut b) = (Float::floor((x + w-1.0) / self.tile_size) as int, Float::floor((y + h-1.0) / self.tile_size) as int);

            // Wrapping
            if r >= self.width as int { r -= self.width as int }
            if b >= self.height as int { b -= self.height as int }

            (r, b)
        };

        ((left, top), (right, bottom))
    }

    fn get_tiles_in_rect(&self, rect: &Rect<f32>) -> ([(&Tile, int, int), ..4], (int, int), (int, int)) {
        let ((left, top), (right, bottom)) = self.get_left_top_tile_coord(rect);

        ([
            (self.get_tile(left as u8, top as u8), left, top),
            (self.get_tile(right as u8, top as u8), right, top),
            (self.get_tile(left as u8, bottom as u8), left, bottom),
            (self.get_tile(right as u8, bottom as u8), right, bottom),
        ], (left, top), (right, bottom))
    }

    pub fn is_tile_inside(&self, rect: &Rect<f32>, tile_id: u16) -> Option<(u8, u8)> {
        let (tiles, _left_top, _right_bottom) = self.get_tiles_in_rect(rect);
        for &(tile, x, y) in tiles.iter() {
            if (*tile).tile_type.id-1 == tile_id {
                return Some((x as u8, y as u8))
            }
        }
        None
    }

    pub fn is_dirt_entrance_below(&self, rect: &Rect<f32>) -> Option<(u8, u8)> {
        self.is_tile_inside(&rect.offset(&self.screen, 0.0, self.tile_size / 4.0), 0x15)
    }

    pub fn is_key_entrance_beside(&self, rect: &Rect<f32>) -> Option<(u8, u8)> {
        if let Some((x, y)) = self.is_tile_inside(&rect.offset(&self.screen, -self.tile_size / 4.0, 0.0), 0x17) {
            Some((x, y))
        } else {
            None
        }
    }

    pub fn remove_key_entrance(&mut self, x: u8, y: u8) {
        self.set_tile(x, y, Tile::empty());
    }

    pub fn has_non_blocking_tile(&self, rect: &Rect<f32>) -> Option<(u8, u8)> {
        let (tiles, _left_top, _right_bottom) = self.get_tiles_in_rect(rect);
        for &(tile, x, y) in tiles.iter() {
            if !(*tile).tile_type.is_blocking {
                return Some((x as u8, y as u8))
            }
        }
        None
    }

    pub fn take_coins(&mut self, rect: &Rect<f32>) -> uint {
        let mut count = 0;
        while let Some((x, y)) = self.is_tile_inside(rect, 0x20) {
            self.set_tile(x, y, Tile::empty());

            count += 1;
        }

        count
    }
}

pub struct Level {
    pub width: u8,
    pub height: u8,
    pub player_start_pos: (f32, f32),
    tiles: Tiles,
    pub switches: Vec<Switch>,
    pub chests: Vec<Chest>,
    pub beanstalks: Vec<Beanstalk>,
    pub monsters1: Vec<Monster1>,
    pub monsters2: Vec<Monster2>,
    pub sticky_keys: Vec<StickyKey>,
    pub messages: Vec<Message>,
    pub set_tos: Vec<SetTo>
}

impl Level {
    pub fn load() -> Level {
        let level_data = include_str!("../../assets/level.json");

        parse_from_json(level_data)
    }

    pub fn get_tiles(&self) -> &Tiles { &self.tiles }

    pub fn get_tiles_mut(&mut self) -> &mut Tiles { &mut self.tiles }

    pub fn iter(&self) -> LevelTileIterator {
        self.tiles.iter()
    }

    pub fn get_screen(&self) -> Screen {
        Screen::new(self.width as f32 * 16.0, self.height as f32 * 16.0)
    }

    pub fn level_size_as_u32(&self) -> (u32, u32) {
        (self.width as u32 * 16, self.height as u32 * 16)
    }

    pub fn trigger_set_to(&mut self, trigger: u8) -> bool {
        let mut did_something = false;
        for set_to in self.set_tos.iter().filter(|s| s.triggered_by == Some(trigger)) {
            self.tiles.apply_set_to(set_to);
            did_something = true;
        }

        did_something
    }
}

pub struct LevelTileIterator<'a> {
    tiles: &'a Vec<Tile>,
    width: uint,
    index: uint
}

impl<'a> Iterator<(u8, u8, &'a Tile)> for LevelTileIterator<'a> {
    fn next(&mut self) -> Option<(u8, u8, &'a Tile)> {
        if self.index >= self.tiles.len() {
            None
        } else {
            let tile = self.tiles.index(&self.index);
            let (x, y) = (self.index % self.width, self.index / self.width);
            self.index += 1;

            Some((x as u8, y as u8, tile))
        }
    }
}

fn parse_from_json(input: &str) -> Level {
    use std::str::FromStr;
    use serialize::json::Json;


    let (width, height) = (28, 16);
    let tile_size: f32 = 16.0;

    let json = match FromStr::from_str(input) {
        Some(Json::Object(obj)) => obj,
        _ => panic!("Not a JSON object")
    };

    let layers = json.get("layers").unwrap().as_array().expect("Not a JSON array");

    // First layer is the level data
    let level_data_json = {
        let layer = layers[0].as_object().expect("Not a JSON object");
        layer.get("data").unwrap().as_array().expect("Not a JSON array")
    };

    let tiles_vec: Vec<Tile> = level_data_json.iter().map(|num_json| {
        let value = num_json.as_u64().expect("Not a JSON number") as u32;

        let (id, flip_x, flip_y) = (value & 0x3FFFFFFF, (value & 0x80000000) != 0, (value & 0x40000000) != 0);

        Tile {
            tile_type: TileType::from_id(id as u16),
            flip_x: flip_x,
            flip_y: flip_y
        }
    }).collect();

    assert_eq!(tiles_vec.len(), width*height);

    let objects_json = {
        let layer = layers[1].as_object().expect("Not a JSON object");
        layer.get("objects").unwrap().as_array().expect("Not a JSON array")
    };

    let mut player_start_pos = (0.0, 0.0);
    let mut switches: Vec<Switch> = Vec::new();
    let mut chests: Vec<Chest> = Vec::new();
    let mut beanstalks: Vec<Beanstalk> = Vec::new();
    let mut monsters1: Vec<Monster1> = Vec::new();
    let mut monsters2: Vec<Monster2> = Vec::new();
    let mut sticky_keys: Vec<StickyKey> = Vec::new();
    let mut messages: Vec<Message> = Vec::new();
    let mut set_tos: Vec<SetTo> = Vec::new();

    for x in objects_json.iter() {

        let object = x.as_object().expect("Not a JSON object");
        let x = object.get("x").unwrap().as_f64().unwrap() as f32;
        let y = object.get("y").unwrap().as_f64().unwrap() as f32;
        let width = object.get("width").unwrap().as_f64().unwrap() as f32;
        let height = object.get("height").unwrap().as_f64().unwrap() as f32;
        let properties = object.get("properties").unwrap().as_object().expect("Not a JSON object");
        let typ = object.get("type").unwrap().as_string().unwrap();

        match typ {
            "player" => { player_start_pos = (x, y); },
            "switch" => {
                let trigger = parse_property_as_number(properties, "trigger").expect("Requires 'trigger'");
                let triggered_by = parse_property_as_number(properties, "triggered_by");

                switches.push(Switch {
                    x: x,
                    y: y,
                    trigger: trigger,
                    triggered_by: triggered_by
                });
            },
            "chest" => {
                let trigger = parse_property_as_number(properties, "trigger");
                let explode_trigger = parse_property_as_number(properties, "explode_trigger");
                let triggered_by = parse_property_as_number(properties, "triggered_by");
                let poof = parse_property_as_boolean(properties, "poof");
                let is_static = parse_property_as_boolean(properties, "static");
                let contains = properties.get("contains").expect("Requires 'contains'").as_string().expect("Not a JSON string").to_string();

                chests.push(Chest {
                    x: x,
                    y: y,
                    trigger: trigger,
                    explode_trigger: explode_trigger,
                    triggered_by: triggered_by,
                    poof: poof,
                    is_static: is_static,
                    fall_distance: height - tile_size,
                    contains: contains
                });
            },
            "beanstalk" => {
                let triggered_by = parse_property_as_number(properties, "triggered_by");
                let poof = parse_property_as_boolean(properties, "poof");

                beanstalks.push(Beanstalk {
                    x: x,
                    y: y,
                    height: (height / tile_size) as uint,
                    triggered_by: triggered_by,
                    poof: poof,
                });
            },
            "monster1" => {
                let triggered_by = parse_property_as_number(properties, "triggered_by");

                monsters1.push(Monster1 {
                    x: x,
                    y: y,
                    triggered_by: triggered_by
                });
            },
            "monster2" => {
                let triggered_by = parse_property_as_number(properties, "triggered_by");

                monsters2.push(Monster2 {
                    x: x,
                    y: y,
                    triggered_by: triggered_by
                });
            },
            "stickykey" => {
                sticky_keys.push(StickyKey {
                    x: x,
                    y: y,
                    fall_distance: height - tile_size / 2.0
                })
            },
            "message" => {
                let triggered_by = parse_property_as_number(properties, "triggered_by");

                let message_tiles = parse_tiles(properties, "tiles");
                let w = (width / tile_size) as uint;
                let h = (height / tile_size) as uint;

                assert_eq!(message_tiles.len(), w*h);

                messages.push(Message {
                    x: x,
                    y: y,
                    width: w,
                    height: h,
                    tiles: message_tiles,
                    triggered_by: triggered_by
                })
            },
            "setto" => {
                let triggered_by = parse_property_as_number(properties, "triggered_by");

                let tile_id = parse_property_as_number(properties, "tile").expect("Reqires 'tile'");

                let tile_x = (x / tile_size) as uint;
                let tile_y = (y / tile_size) as uint;
                let w = (width / tile_size) as uint;
                let h = (height / tile_size) as uint;

                set_tos.push(SetTo {
                    x: tile_x,
                    y: tile_y,
                    width: w,
                    height: h,
                    tile: Tile {
                        tile_type: TileType::from_id(tile_id),
                        flip_x: false,
                        flip_y: false
                    },
                    triggered_by: triggered_by
                })
            }
            _ => panic!("Unknown type: {}", typ)
        };
    }

    let tiles = Tiles::new(width as u8, height as u8, tiles_vec, tile_size);

    Level {
        width: width as u8,
        height: height as u8,
        player_start_pos: player_start_pos,
        tiles: tiles,
        switches: switches,
        chests: chests,
        beanstalks: beanstalks,
        monsters1: monsters1,
        monsters2: monsters2,
        sticky_keys: sticky_keys,
        messages: messages,
        set_tos: set_tos
    }
}

fn parse_tiles(properties: &serialize::json::Object, key: &str) -> Vec<u16> {
    match properties.get(key) {
        Some(j) => {
            let value_str = j.as_string().expect("Not a JSON string");

            value_str.split(' ').map(|num_str| {
                FromStrRadix::from_str_radix(num_str, 16).expect("Not a base 16 number")
            }).collect()
        },
        None => panic!("No tiles")
    }
}

fn parse_property_as_boolean(properties: &serialize::json::Object, key: &str) -> bool {
    match properties.get(key) {
        Some(j) => {
            let value_str = j.as_string().expect("Not a JSON string");
            match value_str {
                "true" => true,
                _ => false
            }
        },
        None => false
    }
}

fn parse_property_as_number<T: FromStrRadix>(properties: &serialize::json::Object, key: &str) -> Option<T> {
    match properties.get(key) {
        Some(j) => {
            let value_str = j.as_string().expect("Not a JSON string");
            Some(FromStrRadix::from_str_radix(value_str, 10).expect("Not a base 10 number"))
        },
        None => None
    }
}
