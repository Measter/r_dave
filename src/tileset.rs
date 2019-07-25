use std::{
    fmt::Write,
    ops::Index,
};

use image::{FilterType, Rgba, RgbaImage};
use piston_window::{Texture, TextureSettings, G2dTextureContext, G2dTexture};

use crate::{SCALE, Result};

const NUM_TILES: u8 = 159;

// See the level.rs file comment for the reason behind this data structure.

#[derive(Debug)]
pub struct TileSet(Vec<G2dTexture>);

impl Index<TileId> for TileSet {
    type Output = G2dTexture;
    fn index(&self, idx: TileId) -> &Self::Output {
        // Because we're indexing through a type that can only be constructed
        // by going through validation, we can skip the bounds check here.
        unsafe{ self.0.get_unchecked(idx.0 as usize) }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TileId(u8);

impl TileId {
    pub const TILE_DAVE_RIGHT: TileId = TileId(TileId::TILE_DAVE_RIGHT_FIRST);
    pub const TILE_DAVE_BASIC: TileId = TileId(56);
    pub const TILE_DAVE_LEFT: TileId = TileId(TileId::TILE_DAVE_LEFT_FIRST);
    pub const TILE_DAVE_JETPACK_RIGHT: TileId = TileId(TileId::TILE_DAVE_JETPACK_RIGHT_FIRST);
    pub const TILE_DAVE_JETPACK_LEFT: TileId = TileId(TileId::TILE_DAVE_JETPACK_LEFT_FIRST);
    pub const TILE_DAVE_JUMP_RIGHT: TileId = TileId(67);
    pub const TILE_DAVE_JUMP_LEFT: TileId = TileId(68);

    pub const TILE_BLANK: TileId = TileId(0);
    pub const TILE_GUN: TileId = TileId(20);
    pub const TILE_JETPACK: TileId = TileId(4);
    pub const TILE_BULLET_LEFT: TileId = TileId(128);
    pub const TILE_BULLET_RIGHT: TileId = TileId(127);

    pub const TILE_ENEMY_BULLET_RIGHT: TileId = TileId(TileId::TILE_ENEMY_BULLET_RIGHT_FIRST);
    pub const TILE_ENEMY_BULLET_LEFT: TileId = TileId(TileId::TILE_ENEMY_BULLET_LEFT_FIRST);

    pub const TILE_MONSTER_SPIDER: TileId = TileId(TileId::TILE_ENEMY_SPIDER_FIRST);
    pub const TILE_MONSTER_WHEEL: TileId = TileId(TileId::TILE_ENEMY_WHEEL_FIRST);
    pub const TILE_MONSTER_STAR: TileId = TileId(TileId::TILE_ENEMY_STAR_FIRST);
    pub const TILE_MONSTER_BAR: TileId = TileId(TileId::TILE_ENEMY_BAR_FIRST);
    pub const TILE_MONSTER_FLAT_DISK: TileId = TileId(TileId::TILE_ENEMY_FLAT_DISK_FIRST);
    pub const TILE_MONSTER_MOUTH: TileId = TileId(TileId::TILE_ENEMY_MOUTH_FIRST);
    pub const TILE_MONSTER_GREEN_DISK: TileId = TileId(TileId::TILE_ENEMY_GREEN_DISK_FIRST);
    pub const TILE_MONSTER_BIG_DISK: TileId = TileId(TileId::TILE_ENEMY_BIG_DISK_FIRST);

    pub const TILE_MONSTER_DYING: TileId = TileId(129);

    pub const TILE_UI_SCORE: TileId = TileId(137);
    pub const TILE_UI_LEVEL: TileId = TileId(136);
    pub const TILE_UI_DAVES: TileId = TileId(135);
    pub const TILE_UI_DAVE: TileId = TileId(143);
    pub const TILE_UI_TROPHY: TileId = TileId(138);
    pub const TILE_UI_GUN: TileId = TileId(134);
    pub const TILE_UI_JETPACK: TileId = TileId(133);
    pub const TILE_UI_JETPACK_FUEL_BORDER: TileId = TileId(141);
    pub const TILE_UI_JETPACK_FUEL_BAR: TileId = TileId(142);
    pub const TILE_UI_BORDER: TileId = TileId(158);

    pub const TILE_SCORE_BLUE_GEM: TileId = TileId(47);
    pub const TILE_SCORE_ORB: TileId = TileId(48);
    pub const TILE_SCORE_RED_GEM: TileId = TileId(49);
    pub const TILE_SCORE_CROWN: TileId = TileId(50);
    pub const TILE_SCORE_RING: TileId = TileId(51);
    pub const TILE_SCORE_SCEPTER: TileId = TileId(52);


    // Animation frame info.
    const TILE_FIRE_FIRST: u8 = 6;
    const TILE_FIRE_LAST: u8 = 9;

    const TILE_TROPHY_FIRST: u8 = 10;
    const TILE_TROPHY_LAST: u8 = 14;

    const TILE_WEEDS_FIRST: u8 = 25;
    const TILE_WEEDS_LAST: u8 = 28;

    const TILE_WATER_FIRST: u8 = 36;
    const TILE_WATER_LAST: u8 = 40;

    const TILE_EXPLOSION_FIRST: u8 = 129;
    const TILE_EXPLOSION_LAST: u8 = 132;

    const TILE_DAVE_RIGHT_FIRST: u8 = 53;
    const TILE_DAVE_RIGHT_LAST: u8 = 55;
    const TILE_DAVE_LEFT_FIRST: u8 = 57;
    const TILE_DAVE_LEFT_LAST: u8 = 58;
    const TILE_DAVE_JETPACK_RIGHT_FIRST: u8 = 77;
    const TILE_DAVE_JETPACK_RIGHT_LAST: u8 = 79;
    const TILE_DAVE_JETPACK_LEFT_FIRST: u8 = 80;
    const TILE_DAVE_JETPACK_LEFT_LAST: u8 = 82;

    const TILE_ENEMY_BULLET_RIGHT_FIRST: u8 = 121;
    const TILE_ENEMY_BULLET_RIGHT_LAST: u8 = 123;
    const TILE_ENEMY_BULLET_LEFT_FIRST: u8 = 124;
    const TILE_ENEMY_BULLET_LEFT_LAST: u8 = 126;

    const TILE_ENEMY_SPIDER_FIRST: u8 = 89;
    const TILE_ENEMY_SPIDER_LAST: u8 = 92;
    const TILE_ENEMY_WHEEL_FIRST: u8 = 93;
    const TILE_ENEMY_WHEEL_LAST: u8 = 96;
    const TILE_ENEMY_STAR_FIRST: u8 = 97;
    const TILE_ENEMY_STAR_LAST: u8 = 100;
    const TILE_ENEMY_BAR_FIRST: u8 = 101;
    const TILE_ENEMY_BAR_LAST: u8 = 104;
    const TILE_ENEMY_FLAT_DISK_FIRST: u8 = 105;
    const TILE_ENEMY_FLAT_DISK_LAST: u8 = 108;
    const TILE_ENEMY_MOUTH_FIRST: u8 = 109;
    const TILE_ENEMY_MOUTH_LAST: u8 = 112;
    const TILE_ENEMY_GREEN_DISK_FIRST: u8 = 113;
    const TILE_ENEMY_GREEN_DISK_LAST: u8 = 116;
    const TILE_ENEMY_BIG_DISK_FIRST: u8 = 117;
    const TILE_ENEMY_BIG_DISK_LAST: u8 = 120;

    const TILE_UI_DIGIT_0: u8 = 148;
}

impl TileId {
    pub fn new(id: u8) -> Result<TileId> {
        if id < NUM_TILES {
            Ok(TileId(id))
        } else {
            Err(format!("Invalid tile id: {}", id))?
        }
    }

    pub fn get_frame(self, tick: usize) -> TileId {
        let last_frame = match self.0 {
            TileId::TILE_FIRE_FIRST                 => TileId::TILE_FIRE_LAST,
            TileId::TILE_TROPHY_FIRST               => TileId::TILE_TROPHY_LAST,
            TileId::TILE_WEEDS_FIRST                => TileId::TILE_WEEDS_LAST,
            TileId::TILE_WATER_FIRST                => TileId::TILE_WATER_LAST,
            TileId::TILE_EXPLOSION_FIRST            => TileId::TILE_EXPLOSION_LAST,

            TileId::TILE_DAVE_RIGHT_FIRST           => TileId::TILE_DAVE_RIGHT_LAST,
            TileId::TILE_DAVE_LEFT_FIRST            => TileId::TILE_DAVE_LEFT_LAST,
            TileId::TILE_DAVE_JETPACK_RIGHT_FIRST   => TileId::TILE_DAVE_JETPACK_RIGHT_LAST,
            TileId::TILE_DAVE_JETPACK_LEFT_FIRST    => TileId::TILE_DAVE_JETPACK_LEFT_LAST,

            TileId::TILE_ENEMY_BULLET_RIGHT_FIRST   => TileId::TILE_ENEMY_BULLET_RIGHT_LAST,
            TileId::TILE_ENEMY_BULLET_LEFT_FIRST    => TileId::TILE_ENEMY_BULLET_LEFT_LAST,

            TileId::TILE_ENEMY_SPIDER_FIRST         => TileId::TILE_ENEMY_SPIDER_LAST,
            TileId::TILE_ENEMY_WHEEL_FIRST          => TileId::TILE_ENEMY_WHEEL_LAST,
            TileId::TILE_ENEMY_STAR_FIRST           => TileId::TILE_ENEMY_STAR_LAST,
            TileId::TILE_ENEMY_BAR_FIRST            => TileId::TILE_ENEMY_BAR_LAST,
            TileId::TILE_ENEMY_FLAT_DISK_FIRST      => TileId::TILE_ENEMY_FLAT_DISK_LAST,
            TileId::TILE_ENEMY_MOUTH_FIRST          => TileId::TILE_ENEMY_MOUTH_LAST,
            TileId::TILE_ENEMY_GREEN_DISK_FIRST     => TileId::TILE_ENEMY_GREEN_DISK_LAST,
            TileId::TILE_ENEMY_BIG_DISK_FIRST       => TileId::TILE_ENEMY_BIG_DISK_LAST,
            _ => return self,
        };

        let tile_offset = (tick / 5) % (last_frame - self.0 + 1) as usize;

        TileId(self.0 + tile_offset as u8)
    }

    pub fn is_collidable(self) -> bool {
        match self.0 {
            1 | 3 | 5 | 15..=19 | 21..=24 | 29 | 30 => true,
            _ => false,
        }
    }

    pub fn is_hazard(self) -> bool {
        match self.0 {
            6..=9 | 25..=28 | 36..=40 => true,
            _ => false
        }
    }

    pub fn is_pickup(self) -> bool {
        match self.0 {
            4 | 10..=14 | 20 | 47..=52 => true,
            _ => false,
        }
    }

    pub fn is_door(self) -> bool {
        self.0 == 2
    }

    pub fn is_trophy(self) -> bool {
        match self.0 {
            10..=14 => true,
            _ => false,
        }
    }

    pub fn get_digit_tile(digit: u32) -> TileId {
        match digit {
            0..=9 => TileId(TileId::TILE_UI_DIGIT_0 + digit as u8),
            _ => panic!("Invalid tile digit"),
        }
    }

    fn is_dave(id: u8) -> bool {
        match id {
            53..=59| 67 | 68 | 71..=73 | 77..=82 => true,
            _ => false,
        }
    }

    fn black_mask(id: u8) -> bool {
        match id {
            89 ..=120 | 129..=132 | 142 => true,
            _ => false,
        }
    }

    fn get_dave_mask(id: u8) -> u8 {
        match id {
            53..=59 => id + 7,
            67 | 68 => id + 2,
            71..=73 => id + 3,
            77..=82 => id + 6,
            _ => panic!("Invalid Dave tile!"),
        }
    }
}

pub fn load_tileset(mut context: G2dTextureContext) -> Result<TileSet> {
    let mut tiles = Vec::new();

    let mut name_buf = String::new();
    for i in 0..NUM_TILES-1 {
        name_buf.clear();
        write!(&mut name_buf, "tiles/tile{}.bmp", i)?;

        let mut tile = image::open(&name_buf)?.to_rgba();

        // Now we apply the alpha mask to the dave tile.
        if TileId::is_dave(i) {
            let mask_id = TileId::get_dave_mask(i);
            name_buf.clear();
            write!(&mut name_buf, "tiles/tile{}.bmp", mask_id)?;
            let mask = image::open(&name_buf)?.to_rgba();

            for ((_, _ , tp), (_, _, mp)) in tile.enumerate_pixels_mut().zip(mask.enumerate_pixels()) {
                if mp == &Rgba([0xfc, 0xfc, 0xfc, 0xff]) {
                    tp[3] = 0;
                }
            }
        } else if TileId::black_mask(i) {
            for (_, _, p) in tile.enumerate_pixels_mut() {
                if p == &Rgba([0, 0, 0, 0xFF]) {
                    p[3] = 0;
                }
            }
        }

        // Because we're doing a scaling here, and the rendering backend doesn't support it,
        // we need to resize the tile image.
        let image = image::imageops::resize(
            &tile,
            tile.width() * SCALE,
            tile.height()* SCALE,
            FilterType::Nearest,
        );

        let texture = Texture::from_image(
            &mut context,
            &image,
            &TextureSettings::new(),
        )?;

        tiles.push(texture);
    }

    // The border image above and below the play area seem to be generated at runtime.
    // We'll do the same here.

    let pixel = |p| match p {
         0..= 7 | 24..=31 => Rgba([101, 101, 101, 255]),
         8..=10 | 21..=23 => Rgba([125, 125, 125, 255]),
        11..=12 | 19..=20 => Rgba([154, 154, 154, 255]),
        13..=13 | 18..=18 => Rgba([182, 182, 182, 255]),
        14..=14 | 17..=17 => Rgba([211, 211, 211, 255]),
        15..=15 | 16..=16 => Rgba([239, 239, 239, 255]),
        _ => Rgba([0, 0, 0, 255]),
    };

    let mut image = RgbaImage::new(32*SCALE, 2*SCALE);
    for (x, _, d) in image.enumerate_pixels_mut() {
        *d = pixel(x/SCALE);
    }

    let texture = Texture::from_image(
        &mut context,
        &image,
        &TextureSettings::new()
    )?;

    tiles.push(texture);

    Ok(TileSet(tiles))
}