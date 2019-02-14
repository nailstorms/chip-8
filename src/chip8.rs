extern crate rand;

use std::io::{Read, Write, BufWriter};
use std::slice::Chunks;

use rand::Rng;

/// Size of the RAM in bytes
const RAM_SIZE: usize = 4096;
/// Depth of the stack
const STACK_SIZE: usize = 16;
/// Number of data registers, i.e. `V0` .. `VF`
const DATA_REGISTERS_COUNT: usize = 16;
/// Memory address for program (ROM) start
const PROGRAM_START: u16 = 0x200;

/// Memory address of built-in font sprites
const FONT_ADDR: usize = 0;
/// Number of rows in one font sprite
const FONT_HEIGHT: usize = 5;
/// Size of one font sprite
const FONT_BYTES: usize = FONT_HEIGHT * 16;
/// Data of the built-in font
const FONT: [u8; FONT_BYTES] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
/// Width of the screen in pixels
const SCREEN_WIDTH: usize = 64;
/// Height of the screen in pixels
const SCREEN_HEIGHT: usize = 32;
/// Total number of pixels of the screen
const SCREEN_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

/// Number of keys on the keypad
const KEYS_COUNT: usize = 16;

/// Virtual machine
///
/// The virtual machine manages state like its registers,
/// RAM, stack, screen pixels, pressed keys as well as
/// timers and some internal state.
pub struct Vm {
    opcode: u16,
    ram: [u8; RAM_SIZE],

    /// CPU registers (V0-VF)
    v: [u8; DATA_REGISTERS_COUNT],

    /// Index register and program counter
    i: u16,
    pc: u16,

    /// Screen: 64 x 32 pixels
    pub screen: [u8; SCREEN_PIXELS],

    /// HEX based keypad (0x0-0xF)
    keys: [u8; KEYS_COUNT],

    /// Timer registers
    delay_timer: u8,
    sound_timer: u8,

    /// Stack and stack pointer
    stack: [u16; STACK_SIZE],
    sp: u16,

    pub draw_flag: bool,
}

impl Vm {
    /// Creates a new `Vm` instance with default state
    pub fn new() -> Vm {

        /*
        let mut memory: [u8; 4096] = [0; 4096];

        for i in 0..80 {
            memory[i] = FONT[i];
        }
        */

        let mut vm = Vm {

            pc: PROGRAM_START,
            i: 0,
            opcode: 0,
            v: [0; DATA_REGISTERS_COUNT],
            ram: memory,

            // inputs/outputs
            screen: [0; SCREEN_PIXELS],
            keys: [0; KEYS_COUNT],

            // initialize stack and stack pointer
            stack: [0; STACK_SIZE],
            sp: 0,

            // reset timers
            delay_timer: 0,
            sound_timer: 0,

            draw_flag: true,
        };

        {
            let mut ram = BufWriter::new(&mut vm.ram[FONT_ADDR..(FONT_ADDR + FONT_BYTES)]);
            ram.write_all(FONT.AsRef()).unwrap();
            debug!("Initialized VM with built-in font");
        }
        vm

    }

    pub fn load_game(&mut self, game: &str) {
        //create a path to the desired file
        let path = Path::new(game);
        let display = path.display();

        // open the path in read-only mode, returns
        let mut file = match File::open(&path) {
            // the 'description' method of 'io:Error' returns a string that describes the error
            Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
            Ok(file) => file,
        };

        // Read the file
        let mut buffer = Vec::new();
        match file.read_to_end(&mut buffer) {
            Err(why) => panic!("couldn't read {}: {}", display, Error::description(&why)),
            Ok(_) => println!("{} contains:\n{} bytes", display, buffer.len()),
        };

        let buffer_size = buffer.len();

        // load the game into memory
        for i in 0..buffer_size {
            self.ram[i + 512] = buffer[i];
        }
    }

    pub fn emulate_cycle(&mut self) {
        //fetch opcode: merge two memory locations to for an opcode
        self.opcode = (self.ram[self.pc as usize] as u16) << 8 | self.ram[self.pc as usize + 1] as u16;

        // register identifiers
        let x = ((self.opcode & 0x0F00) as usize) >> 8;
        let y = ((self.opcode & 0x00F0) as usize) >> 4;

        // constants
        // let n = self.opcode & 0x000F; //u16
        let nn = self.opcode & 0x00FF; // u16

        // addr
        let nnn = self.opcode & 0x0FFF; // u16

        println!("Executing opcode 0x{:X}", self.opcode);

        // decode opcode & execute opcode
        match self.opcode & 0xF000 {
            0x0000 => match self.opcode & 0x000F {
                // 00E0: clears the screen
                0x0000 => {
                    self.screen = [0; 64 * 32];
                    self.draw_flag = true;
                    self.pc += 2;
                },
                // 00EE: Returns from subroutine
                0x000E => {
                    self.sp -= 1; // pop the stack
                    self.pc = self.stack[self.sp as usize];
                    self.pc += 2;
                },
                _ => println!("Unknown opcode [0x0000]: {:X}", self.opcode),
            },
            // 1NNN = Jumps to address NNN.
            0x1000 => {
                self.pc = nnn;
            },
            // 2NNN = calls subroutine at NNN
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            },
            // 3XNN = Skips the next instruction if VX equals NN.
            0x3000 => {
                if self.v[x] == (nn as u8) {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },
            // 4XNN = Skips the next instruction if VX doesn't equal NN.
            0x4000 => {
                if self.v[x] != (nn as u8) {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },

            //* 5XY0 = Skips the next instruction if VX equals VY.
            0x5000 => {
                if self.v[x] == self.v[y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },
            //* 6XNN = Sets VX to NN
            0x6000 => {
                self.v[x] = nn as u8;
                self.pc += 2;
            },
            //* 7XNN = Adds NN to VX
            0x7000 => {
                self.v[x] += nn as u8;
                self.pc += 2;
            },

            0x8000 => match self.opcode & 0x000F {
                //* 8XY0 = Sets VX to the value of VY
                0x0000 => {
                    self.v[x] = self.v[y];
                    self.pc += 2;
                },
                //* 8XY1 = Sets VX to VX or VY.
                0x0001 => {
                    self.v[x] = self.v[x] | self.v[y];
                    self.pc += 2;
                },
                //* 8XY2 = Sets VX to VX and VY.
                0x0002 => {
                    self.v[x] = self.v[x] & self.v[y];
                    self.pc += 2;
                },
                //* 8XY3 = Sets VX to VX xor VY.
                0x0003 => {
                    self.v[x] = self.v[x] ^ self.v[y];
                    self.pc += 2;
                },

                //* 8XY4 = Adds VY to VX. VF is set to 1 when there's a carry,
                // and to 0 when there isn't.
                0x0004 => { // 8XY4 = add the value of VY to VX
                    if self.v[y] > (0xFF - self.v[x]) {
                        self.v[0xF] = 1; // carry
                    } else {
                        self.v[0xF] = 0;
                    }
                    self.v[x] += self.v[y];

                    self.pc += 2;
                },
                0x0005 => { //* 8XY5 =
                    if self.v[y] > self.v[x] {
                        self.v[0xF] = 0 // borrow
                    } else {
                        self.v[0xF] = 1;
                    }
                    self.v[x] -= self.v[y];
                    self.pc += 2;
                },

                0x0006 => { //* 8XY6 =
                    // Shifts VX right by one. VF is set to the value of
                    // the least significant bit of VX before the shift
                    let lsb_vx = (self.v[x] << 7) >> 7;
                    self.v[0xF] = lsb_vx;
                    self.v[x] >>= 1;
                    self.pc += 2;
                },

                0x0007 => { //* 8XY7 =
                    if self.v[y] < self.v[x] {
                        self.v[0xF] = 0; // borrow
                    } else {
                        self.v[0xF] = 1;
                    }
                    self.v[x] = self.v[y] - self.v[x];
                    self.pc += 2;
                },

                //* 8XYE = Shifts VX left by one. VF is set to the value of
                // the most significant bit of VX before the shift
                0x000E => {
                    let msb_vx = self.v[x] >> 7;
                    self.v[0xF] = msb_vx;
                    self.v[x] <<= 1;
                    self.pc += 2;
                },

                _ => println!("Unknown opcode [0x8000]: {:02X}", self.opcode),
            },

            // 9XY0 = Skips the next instruction if VX doesn't equal VY.
            0x9000 => {
                if self.v[x] != self.v[y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },

            // ANNN = Sets I to the address NNN.
            0xA000 => {
                self.i = nnn;
                self.pc += 2;
            },

            // * BNNN = jumps to the address NNN plus V0.
            0xB000 => {
                self.pc = nnn + (self.v[0x0] as u16);
            },

            //* CXNN = Set VX to a random number, masked by NN.
            0xC000 => {
                // generate a random u8
                let mut rng = rand::thread_rng();
                let random_number = rng.gen::<u8>();

                // set vx to a random number, masked by nn
                self.v[x] = random_number & (nn as u8);

                self.pc += 2;
            }
        };
    }

    pub fn set_keys(&self) {

    }
}