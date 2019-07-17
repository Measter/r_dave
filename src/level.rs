use std::{
    fmt::Write as FmtWrite,
    fs::File,
    io::BufReader,
    ops::Index,
};

use byteorder::ReadBytesExt;

use crate::{
    Result,
    tileset::TileId,
    game::Position,
    monster::Monster,
};
use std::ops::IndexMut;

const NUM_LEVELS: usize = 10;
const MONSTER_PATH_LEN: usize = 128;

// Because the level list will be accessed on every frame, I've opted to bypass the
// bounds check. To ensure that the access is still safe, i've wrapped the vector of
// Level structures, and the index into that vector, in a newtype.

// Because I've left the fields of those newtypes private, the only place they can be
// accessed directly is in this module, meaning the types can only be constructed in
// here.

// Due to that restriction, we only need to ensure that the index is valid in this
// file, and not in every single access of the level vector.

pub struct Levels(Vec<Level>);

impl Index<LevelId> for Levels {
    type Output = Level;
    fn index(&self, idx: LevelId) -> &Self::Output {
        // We're indexing through a type that can only be constructed by going
        // through validation, so we can skip bounds checking.
        unsafe { self.0.get_unchecked(idx.0) }
    }
}

impl IndexMut<LevelId> for Levels {
    fn index_mut(&mut self, idx:LevelId) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(idx.0) }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LevelId(usize);

impl LevelId {
    pub fn next(self) -> Option<LevelId> {
        if self.0 < (NUM_LEVELS - 1) {
            Some(LevelId(self.0 + 1))
        } else {
            None
        }
    }

    pub const fn first_level() -> LevelId {
        LevelId(0)
    }

    pub fn start_position(self) -> Position<u8> {
        match self.0 {
            0 => Position { x: 2, y: 8 },
            1 => Position { x: 1, y: 8 },
            2 => Position { x: 2, y: 5 },
            3 => Position { x: 1, y: 5 },
            4 => Position { x: 2, y: 8 },
            5 => Position { x: 2, y: 8 },
            6 => Position { x: 1, y: 2 },
            7 => Position { x: 2, y: 8 },
            8 => Position { x: 6, y: 1 },
            9 => Position { x: 2, y: 8 },
            _ => unreachable!()
        }
    }

    pub fn monsters(self) -> [Monster; 5] {
        match self.0 {
            2 => [
                Monster::init_live(TileId::TILE_MONSTER_SPIDER1, Position { x: 44, y: 4 }),
                Monster::init_live(TileId::TILE_MONSTER_SPIDER1, Position { x: 59, y: 4 }),
                Monster::init_dead(),
                Monster::init_dead(),
                Monster::init_dead(),
            ],
            3 => [
                Monster::init_live(TileId::TILE_MONSTER_WHEEL1, Position { x: 32, y: 2 }),
                Monster::init_dead(),
                Monster::init_dead(),
                Monster::init_dead(),
                Monster::init_dead(),
            ],
            _ => [Monster::init_dead(), Monster::init_dead(), Monster::init_dead(), Monster::init_dead(), Monster::init_dead()],
        }
    }
}

pub struct Level {
    path: MonsterPath,
    tiles: [TileId; 1000],
}

impl Level {
    pub fn path(&self) -> &MonsterPath {
        &self.path
    }

    pub fn tiles(&self) -> &[TileId] {
        &self.tiles
    }

    pub fn tiles_mut(&mut self) -> &mut [TileId] {
        &mut self.tiles
    }
}

pub struct MonsterPath([Position<i16>; 128]);

impl MonsterPath {
    // The casting feels a bit messy, but at least lets us keep the values as i16 for
    // easy comparison.
    pub const PATH_END: Position<i16> = Position { x: 0xEAu8 as i8 as i16, y: 0xEAu8 as i8 as i16 };
}

impl Index<MonsterPathIndex> for MonsterPath {
    type Output = Position<i16>;
    fn index(&self, idx: MonsterPathIndex) -> &Self::Output {
        // As with the level indexing above, we know that the index has been
        // verified to be in bounds at this point, so we can do an unchecked
        // indexing.
        unsafe { self.0.get_unchecked(idx.0) }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MonsterPathIndex(usize);

impl MonsterPathIndex {
    pub const START: MonsterPathIndex = MonsterPathIndex(0);

    pub fn next(self) -> MonsterPathIndex {
        if self.0 < (MONSTER_PATH_LEN - 1) {
            MonsterPathIndex(self.0 + 1)
        } else {
            MonsterPathIndex::START
        }
    }
}


pub fn load_levels() -> Result<Levels> {
    let mut levels = Vec::new();

    let mut name_buf = String::new();

    for i in 0..NUM_LEVELS {
        name_buf.clear();
        write!(&mut name_buf, "levels/level{}.dat", i)?;

        let file = File::open(&name_buf)?;
        let mut file = BufReader::new(file);

        let mut level = Level {
            path: MonsterPath([Default::default(); 128]),
            tiles: [TileId::TILE_BLANK; 1000],
        };

        for pair in level.path.0.iter_mut() {
            pair.x = file.read_i8()? as i16;
            pair.y = file.read_i8()? as i16;
        }

        for t in level.tiles.iter_mut() {
            *t = TileId::new(file.read_u8()?)?;
        }

        levels.push(level);
    }

    Ok(Levels(levels))
}

