# Dangerous Dave
This project is a Rust translation of MaiZure's [Let's Make: Dangerous Dave](https://github.com/MaiZure/lmdave) project.
Currently contains only the tile and level extractors.

## Build Notes
The project is built using the Rust 1.36 MSVC stable compiler. Current direct dependencies are:

* [ByteOrder 1.3.2](https://crates.io/crates/byteorder)
* [Image 0.21.2](https://crates.io/crates/image)

The tile and level extractors expect the original dave executable to be at `orig/dave.exe`, and for the folders 'tiles' and 'levels' to exist.

The tile extractor can be run using `cargo run --bin tile_ext`, and the level extractor can be run using `cargo run --bin level_ext`.