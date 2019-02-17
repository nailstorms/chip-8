extern crate rand;

use std::io::{Read, Write, BufWriter};
use std::fs::File;
use std::path::Path;
use std::error::Error;

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
            ram: [0; RAM_SIZE],

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
            ram.write_all(FONT.as_ref()).unwrap();
            println!("Initialized VM with built-in font");
        }
        vm

    }

    pub fn load_game(&mut self, game: &str) {
        // Create a path to the desired file
        let path = Path::new(game);
        let display = path.display();

        // Open the path in read-only mode, returns
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

        // Load the game into RAM
        for i in 0..buffer_size {
            self.ram[i + 512] = buffer[i];
        }
    }

    pub fn emulate_cycle(&mut self) {
        // fetch opcode: merge two memory locations for an opcode
        self.opcode = (self.ram[self.pc as usize] as u16) << 8 | self.ram[self.pc as usize + 1] as u16;

        // register identifiers
        let x = ((self.opcode & 0x0F00) as usize) >> 8;
        let y = ((self.opcode & 0x00F0) as usize) >> 4;

        // constants
        // value
        let nn = self.opcode & 0x00FF; // u16

        // address
        let nnn = self.opcode & 0x0FFF; // u16

        println!("Executing opcode 0x{:X}", self.opcode);

        // decode opcode & execute opcode
        match self.opcode & 0xF000 {
            0x0000 => match self.opcode & 0x000F {

                // 00E0 =
                // Clear the screen.
                0x0000 => {
                    self.screen = [0; SCREEN_PIXELS];
                    self.draw_flag = true;
                    self.pc += 2;
                },

                // 00EE =
                // Return from subroutine.
                0x000E => {
                    self.sp -= 1; // pop the stack
                    self.pc = self.stack[self.sp as usize];
                    self.pc += 2;
                },
                _ => println!("Unknown opcode [0x0000]: {:X}", self.opcode),
            },

            // 1NNN =
            // Jump to address NNN.
            0x1000 => {
                self.pc = nnn;
            },

            // 2NNN =
            // Call subroutine at NNN
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            },

            // 3XNN =
            // Skip the next instruction if VX equals NN.
            0x3000 => {
                if self.v[x] == (nn as u8) {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },

            // 4XNN =
            // Skip the next instruction if VX isn't equal to NN.
            0x4000 => {
                if self.v[x] != (nn as u8) {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },

            // 5XY0 =
            // Skip the next instruction if VX equals VY.
            0x5000 => {
                if self.v[x] == self.v[y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },

            // 6XNN =
            // Set VX to NN.
            0x6000 => {
                self.v[x] = nn as u8;
                self.pc += 2;
            },

            // 7XNN =
            // Add NN to VX.
            0x7000 => {
                self.v[x] = self.v[x].wrapping_add(nn as u8);
                self.pc += 2;
            },

            0x8000 => match self.opcode & 0x000F {

                // 8XY0 =
                // Set VX to the value of VY
                0x0000 => {
                    self.v[x] = self.v[y];
                    self.pc += 2;
                },

                // 8XY1 =
                // Set VX to VX or VY.
                0x0001 => {
                    self.v[x] = self.v[x] | self.v[y];
                    self.pc += 2;
                },

                // 8XY2 =
                // Set VX to VX and VY.
                0x0002 => {
                    self.v[x] = self.v[x] & self.v[y];
                    self.pc += 2;
                },

                // 8XY3 =
                // Set VX to VX xor VY.
                0x0003 => {
                    self.v[x] = self.v[x] ^ self.v[y];
                    self.pc += 2;
                },

                // 8XY4 =
                // Adds VY to VX. VF is set to 1 when there's a carry,
                // and to 0 when there isn't.
                0x0004 => { // 8XY4 = add the value of VY to VX
                    if self.v[y] > (0xFF - self.v[x]) {
                        self.v[0xF] = 1; // carry
                    } else {
                        self.v[0xF] = 0;
                    }
                    self.v[x] = self.v[x].wrapping_add(self.v[y]);

                    self.pc += 2;
                },

                // 8XY5 =
                // Subtracts VY from VX. VF is set to 1 when there's no borrow,
                // and to 0 when there is. Result is stored in VX.
                0x0005 => {
                    if self.v[y] > self.v[x] {
                        self.v[0xF] = 0 // borrow
                    } else {
                        self.v[0xF] = 1;
                    }
                    self.v[x] -= self.v[y];
                    self.pc += 2;
                },

                // 8XY6 =
                // Shifts VX right by one. VF is set to the value of
                // the least significant bit of VX before the shift
                0x0006 => {
                    let lsb_vx = (self.v[x] << 7) >> 7;
                    self.v[0xF] = lsb_vx;
                    self.v[x] >>= 1;
                    self.pc += 2;
                },

                // 8XY7 =
                // Subtracts VX from VY. VF is set to 1 when there's no borrow,
                // and to 0 when there is. Result is stored in VX.
                0x0007 => {
                    if self.v[y] < self.v[x] {
                        self.v[0xF] = 0; // borrow
                    } else {
                        self.v[0xF] = 1;
                    }
                    self.v[x] = self.v[y] - self.v[x];
                    self.pc += 2;
                },

                // 8XYE =
                // Shifts VX left by one. VF is set to the value of
                // the most significant bit of VX before the shift
                0x000E => {
                    let msb_vx = self.v[x] >> 7;
                    self.v[0xF] = msb_vx;
                    self.v[x] <<= 1;
                    self.pc += 2;
                },

                _ => println!("Unknown opcode [0x8000]: {:02X}", self.opcode),
            },

            // 9XY0 =
            // Skip the next instruction if VX isn't equal to VY.
            0x9000 => {
                if self.v[x] != self.v[y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },

            // ANNN =
            // Set I to the address NNN.
            0xA000 => {
                self.i = nnn;
                self.pc += 2;
            },

            // BNNN =
            // Lump to the address NNN plus V0.
            0xB000 => {
                self.pc = nnn + (self.v[0x0] as u16);
            },

            // CXNN =
            // Set VX to a random number, masked by NN.
            0xC000 => {
                // generate a random u8
                let mut rng = rand::thread_rng();
                let random_number = rng.gen::<u8>();

                // set vx to a random number, masked by nn
                self.v[x] = random_number & (nn as u8);

                self.pc += 2;
            },

            // DXYN =
            // Display n-byte sprite starting at memory location I at (Vx, Vy),
            // Set VF = collision.
            0xD000 => {
                let x = self.v[x] as u16;
                let y = self.v[y] as u16;
                let height = self.opcode & 0x000F; // u16
                let mut pixel: u16;

                self.v[0xF] = 0;
                for yline in 0..height {

                    // get byte
                    pixel = self.ram[(self.i + yline) as usize] as u16;

                    // for each pixel on this line
                    for xline in 0..8 {
                        // check if the current pixel will be drawn by AND-ING it to 1 - IOW
                        // check if the pixel is set to 1 (This will scan through the byte,
                        // one bit at the time)
                        if (pixel & (0x80 >> xline)) != 0 {
                            // since the pixel will be drawn, check the destination location in
                            // gfx for collision (verify if that location is flipped on (== 1))
                            if self.screen[(x + xline + (y + yline) * 64) as usize] == 1 {
                                self.v[0xF] = 1; // register the collision
                            }

                            // draw in XOR mode
                            self.screen[(x + xline + (y + yline) * 64) as usize] ^= 1;
                        }
                    }
                }

                self.draw_flag = true;
                self.pc += 2;
            },

            0xE000 => match self.opcode & 0x00FF {
                // EX9E =
                // Skips the next instruction if the key stored in VX
                // is pressed.
                0x009E => {
                    if self.keys[self.v[x] as usize] != 0 {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                },

                // EXA1 =
                // Skip the next instruction if the key stored in VX
                // isn't pressed.
                0x00A1 => {
                    if self.keys[self.v[x] as usize] != 1 {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                },

                _ => println!("Unknown opcode [0xE000]: {:02X}", self.opcode),
            },

            0xF000 => match self.opcode & 0x00FF {

                // FX07 =
                // Set VX to the value of the delay timer
                0x0007 => {
                    self.v[x] = self.delay_timer;
                    self.pc += 2;
                },

                // FX0A =
                // A key press is awaited, and then stored in VX.
                0x000A => {
                    // TODO: Waits a keypress and stores it in VX
                    let mut key_press = false;
                    for i in 0..16 {
                        if self.keys[i] != 0 {
                            self.v[x] = i as u8;
                            key_press = true;
                        }
                    }

                    // if we didn't receive a keypress, skip this cycle and try again.
                    if !key_press {
                        return;
                    }

                    self.pc += 2;
                },

                // FX15 =
                // Set the delay timer to VX.
                0x0015 => {
                    self.delay_timer = self.v[x];
                    self.pc += 2;
                },

                // FX18 =
                // Set the sound timer to VX.
                0x0018 =>{
                    self.sound_timer = self.v[x];
                    self.pc += 2;
                },

                // FX1E =
                // Add VX to I.
                0x001E => {
                    // TODO: carry bit
                    self.i += self.v[x] as u16;
                    self.pc += 2;
                },
                // FX29 =
                // Set I to the location of the sprite for the character in V.
                0x0029 => {
                    // each character contains 5 elements (reason for * 0x5)
                    self.i = (self.v[x] * 0x5) as u16;
                    self.pc += 2;
                },

                // FX33 =
                // Store binary-coded decimal representation of a value
                // contained in VX to addr i, i+1, and i+2.
                0x0033 => {
                    let i = self.i as usize;

                    self.ram[i] = self.v[x] / 100;
                    self.ram[i + 1] = (self.v[x] / 10) % 10;
                    self.ram[i + 2] = (self.v[x] % 100) % 10;

                    self.pc += 2;
                },

                // FX55 =
                // Store values contained in V0-VX in memory
                // starting at address I.
                0x0055 => {
                    for index in 0..x+1 {
                        self.ram[(self.i as usize) + index] = self.v[index];
                    };
                    self.pc += 2;
                },

                // FX65 =
                // Fills V0-VX with values from memory
                // starting at address I.
                0x0065 => {
                    for index in 0..x+1 {
                        self.v[index] = self.ram[(self.i as usize) + index];
                    };
                    self.pc += 2;
                },

                _ => println!("Unknown opcode [0xF000]: {:02X}", self.opcode),
            },

            _ => println!("Unknown opcode: {:02X}", self.opcode),
        };

        // update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP!");
            }
            self.sound_timer -= 1;
        }
    }

    pub fn set_keys(&self) {

    }
}