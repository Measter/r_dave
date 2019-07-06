use std::{
    path::Path,
    fs::File,
    io::{Read, BufReader, Seek, SeekFrom},
    error::Error,
    iter,
};
use byteorder::{ReadBytesExt, LittleEndian};
use image::{Rgb, RgbImage};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const VGA_DATA_ADDR: u64 = 0x120f0;
const VGA_PAL_ADDR: u64 = 0x26b0a;


fn read_vga_data<R: Read + Seek>(mut file: R) -> Result<Vec<u8>> {
    print!("Reading VGA data...");
    file.seek(SeekFrom::Start(VGA_DATA_ADDR))?;

    // Undo RLE and read all pixel data.
    // Read file length - first 4 bytes LE.
    let final_length = file.read_u32::<LittleEndian>()? as usize;
    let mut raw_data = Vec::<u8>::with_capacity(final_length);

    // Read each byte and decode.
    while raw_data.len() < final_length {
        let mut byte = file.read_u8()?;

        match byte & 0x80 {
            0 => {
                byte += 3;
                let next = file.read_u8()?;

                raw_data.extend(iter::repeat(next).take(byte as usize));
            },
            _ => {
                byte &= 0x7F;
                byte += 1;

                for b in (&mut file).bytes().take(byte as usize) {
                    raw_data.push(b?);
                }
            }
        }
    }

    println!("done");
    Ok(raw_data)
}

fn read_vga_palette<R: Read + Seek> (mut file: R) -> Result<Vec<Rgb<u8>>> {
    print!("Reading VGA palette...");
    // Read in VGA palette. 256-colours of 3 bytes (RGB).
    file.seek(SeekFrom::Start(VGA_PAL_ADDR))?;

    let mut palette = Vec::with_capacity(256);

    for _ in 0..256 {
        palette.push(
            Rgb([
                file.read_u8()? << 2,
                file.read_u8()? << 2,
                file.read_u8()? << 2,
            ])
        )
    }

    println!("done");
    Ok(palette)
}

fn read_tile_indices(mut raw_data: &[u8]) -> Result<Vec<u32>> {
    print!("Reading tile indices...");

    let tile_count = raw_data.read_u32::<LittleEndian>()?;
    let mut tile_indices = Vec::with_capacity(tile_count as usize);

    for _ in 0..tile_count {
        let ti = raw_data.read_u32::<LittleEndian>()?;
        tile_indices.push(ti);
    }

    println!("done");
    Ok(tile_indices)
}

fn make_tiles(raw_data: &[u8], palette: &[Rgb<u8>], indices: &[u32]) -> Result<()> {
    print!("Saving tiles...");

    for (current_tile, current_byte) in indices.iter().enumerate() {
        let mut current_byte = *current_byte as usize;

        // Skip unusual byte.
        if current_byte > 65280 {
            current_byte += 1;
        }

        let (tile_width, tile_height, current_byte) = match &raw_data[current_byte..current_byte+4] {
            [x, 0, y, 0] if (1..0xbf).contains(x) && (1..0x64).contains(y) => {
                (*x as u32, *y as u32, current_byte + 4)
            },
            // Default width of 16x16.
            _ => (16, 16, current_byte)
        };

        let mut surface = RgbImage::new(tile_width, tile_height);

        // Go through the data, matching to palette and writing to surface.
        raw_data[current_byte..].iter()
            .map(|&b| palette[b as usize])
            .zip(surface.pixels_mut())
            .for_each(|(c, p)| *p = c);

        let name = format!("tiles/tile{}.bmp", current_tile);
        surface.save(&name)?;
    }

    println!("done");

    Ok(())
}

fn main() -> Result<()> {
    let file = File::open("orig/dave.exe")?;
    let mut file = BufReader::new(file);

    let raw_data = read_vga_data(&mut file)?;
    let palette = read_vga_palette(&mut file)?;
    let tile_index = read_tile_indices(&raw_data)?;
    make_tiles(&raw_data, &palette, &tile_index)?;

    Ok(())
}