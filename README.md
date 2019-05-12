# chip-8

**chip-8** is a CHIP-8 emulator/interpreter written using the Rust programming language.

## Prerequisites

* [Rust](https://github.com/rust-lang/rust)
* [Cargo](https://crates.io) - Rust's package manager, necessary for building and running the program
* [SDL2](https://github.com/Rust-SDL2/rust-sdl2) - library/crate used for organizing the UI
* Any CHIP-8 ROM that you can find on the Internet ([example](https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html))
* The one and only **chip-8** interpreter (downloaded by `git clone`'ing this repo)

All installation guides are provided in respective links.

## Usage

To run a ROM, pass the location of the ROM file as a command line argument on `cargo run`.

```
cargo run [path-to-ROM]
```

Alternatively, you can build the release version with `cargo build --release` and then launch the executable from target directory; method of passing the ROM is still the same.

```
cargo build --release
cd /target/release
./chip-8 [path-to-ROM]
```

## Documentation

* Wikipedia article - https://en.wikipedia.org/wiki/CHIP-8
* [mattmik : Mastering CHIP-8](http://mattmik.com/files/chip8/mastering/chip8.html)
* [Cowgod's Chip-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
* [Laurence Muller's guide on how to write a CHIP-8 interpreter](http://www.multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/)
* The Rust Programming Language 'book' - https://doc.rust-lang.org/book/index.html
* Cargo docs - https://doc.rust-lang.org/cargo/
* SDL2 docs - https://rust-sdl2.github.io/rust-sdl2/sdl2/
