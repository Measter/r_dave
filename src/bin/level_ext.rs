use std::{
    fs::File,
    io::{Read, BufReader, Seek, SeekFrom, BufWriter, Write as IoWrite},
    error::Error,
    fmt::Write as FmtWrite,
};
use image::{RgbImage, GenericImage};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const LEVEL_ADDR: u64 = 0x26e0a;

struct DaveLevel {
    path: [u8; 256],
    tiles: [u8; 1000],
    padding: [u8; 24],
}

impl Default for DaveLevel {
    fn default() -> DaveLevel {
        DaveLevel {
            path: [0; 256],
            tiles: [0; 1000],
            padding: [0; 24],
        }
    }
}

fn extract_levels() -> Result<Vec<DaveLevel>> {
    print!("Reading levels...");

    let file = File::open("orig/dave.exe")?;
    let mut file = BufReader::new(file);
    file.seek(SeekFrom::Start(LEVEL_ADDR))?;

    let mut levels: Vec<DaveLevel> = Vec::new();
    levels.resize_with(10, Default::default);

    let mut name_buf = String::new();
    // Stream level data to memory and output files.
    for (index, level) in levels.iter_mut().enumerate() {
        write!(&mut name_buf, "levels/level{}.dat", index)?;
        let level_file = File::create(&name_buf)?;
        let mut level_file = BufWriter::new(level_file);

        // Stream path data.
        file.read_exact(&mut level.path)?;
        level_file.write_all(&level.path)?;

        // Stream tile indices.
        file.read_exact(&mut level.tiles)?;
        level_file.write_all(&level.tiles)?;

        // Stream padding.
        file.read_exact(&mut level.padding)?;
        level_file.write_all(&level.padding)?;

        name_buf.clear();
    }

    println!("done");

    Ok(levels)
}

fn load_tiles() -> Result<Vec<RgbImage>> {
    print!("Loading tiles...");
    let mut tiles = Vec::with_capacity(158);

    let mut name_buf = String::new();
    for i in 0..158 {
        write!(&mut name_buf, "tiles/tile{}.bmp", i)?;

        let image = image::open(&name_buf)?;
        tiles.push(image.to_rgb());

        name_buf.clear();
    }

    println!("done");
    Ok(tiles)
}

fn create_level_map(levels: &[DaveLevel], tiles: &[RgbImage]) -> Result<()> {
    print!("Creating world map...");
    let mut world_map = RgbImage::new(1600, 1600);

    for (l_index, level) in levels.iter().enumerate() {
        let tile_iter = level.tiles.iter()
            .enumerate()
            .map(|(i, t)| {
                (i / 100, i % 100, t)
            });

        for (y, x, &tile_index) in tile_iter {
            let dest_x = x*16;
            let dest_y = l_index*160 + y*16;
            let tile = &tiles[tile_index as usize];
            world_map.copy_from(tile, dest_x as u32, dest_y as u32);
        }
    }

    world_map.save("levels/world.png")?;

    println!("done");
    Ok(())
}

fn main() -> Result<()> {
    let levels = extract_levels()?;
    let tiles = load_tiles()?;
    create_level_map(&levels, &tiles)?;

    Ok(())
}