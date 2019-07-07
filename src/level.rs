use std::{
    fmt::Write as FmtWrite,
    fs::File,
    io::BufReader,
    ops::Index,
};

use byteorder::ReadBytesExt;

use crate::Result;
use crate::tileset::TileId;

const NUM_LEVELS: usize = 10;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LevelId(usize);

impl LevelId {
    pub fn next(self) -> LevelId {
        if self.0 < (NUM_LEVELS - 1) {
            LevelId(self.0 + 1)
        } else {
            self
        }
    }

    pub fn prev(self) -> LevelId {
        if self.0 > 0 {
            LevelId(self.0 - 1)
        } else {
            self
        }
    }

    pub const fn first_level() -> LevelId {
        LevelId(0)
    }
}

pub struct Level {
    path: [(i8, i8); 128],
    tiles: [TileId; 1000],
}

impl Level {
    pub fn path(&self) -> &[(i8, i8)] {
        &self.path
    }

    pub fn tiles(&self) -> &[TileId] {
        &self.tiles
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
            path: [(0, 0); 128],
            tiles: [TileId::new(0)?; 1000],
        };

        for pair in level.path.iter_mut() {
            pair.0 = file.read_i8()?;
            pair.1 = file.read_i8()?;
        }

        for t in level.tiles.iter_mut() {
            *t = TileId::new(file.read_u8()?)?;
        }

        levels.push(level);
    }

    Ok(Levels(levels))
}

