extern crate rand;

use std::io::{Read, Write, BufWriter};
use std::fs::File;
use std::path::Path;
use std::error::Error;

use rand::Rng;
use opcodes::*;

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
    pub opcode: u16,
    pub ram: [u8; RAM_SIZE],

    /// CPU registers (V0-VF)
    pub v: [u8; DATA_REGISTERS_COUNT],

    /// Index register and program counter
    pub i: u16,
    pub pc: u16,

    /// Screen: 64 x 32 pixels
    pub screen: [u8; SCREEN_PIXELS],

    /// HEX based keypad (0x0-0xF)
    pub key_states: [bool; KEYS_COUNT],

    /// Timer registers
    pub delay_timer: u8,
    pub sound_timer: u8,

    /// Stack and stack pointer
    pub stack: [u16; STACK_SIZE],
    pub sp: u16,

    pub draw_flag: bool,
}

impl Vm {
    /// Creates a new `Vm` instance with default state
    pub fn init() -> Vm {


        Vm {
            pc: PROGRAM_START,
            i: 0,
            opcode: 0,
            v: [0; DATA_REGISTERS_COUNT],
            ram: [0; RAM_SIZE],

            // inputs/outputs
            screen: [0; SCREEN_PIXELS],
            key_states: [false; KEYS_COUNT],

            // initialize stack and stack pointer
            stack: [0; STACK_SIZE],
            sp: 0,

            // reset timers
            delay_timer: 0,
            sound_timer: 0,

            draw_flag: true,
        }
    }

    pub fn load_font(&mut self) {
        for i in 0..80 {
            self.ram[i] = FONT[i];
        }
    }

    pub fn load_game(&mut self, game_location: &str) {
        // Create a path to the desired file
        let path = Path::new(game_location);
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

    /*
    // Mark the key with index `idx` as being set
    pub fn set_key(&mut self, idx: u8) {
        println!("Set key {}", idx);
        self.keys[idx as usize] = 1;
        if let Some(Vx) = self.key_wait {
            println!("No longer waiting on key");
            self.v[Vx as usize] = idx;
            self.key_wait = None;
        }
    }

    // Reset the key with index `idx`
    pub fn reset_key(&mut self, idx: u8) {
        println!("Reset key {}", idx);
        self.keys[idx as usize] = 0;
    }
    */


    pub fn translate_opcode(&mut self) {

        match self.opcode & 0xF000 {
            0x0000 => match self.opcode & 0x0FFF {

                // 00E0
                0x00E0 => cls(self),
                // 00EE
                0x00EE => ret(self),

                _ => println!("Unknown opcode [0x0000]: {:X}", self.opcode)
            },

            // 1NNN
            0x1000 => jp_addr(self),

            // 2NNN
            0x2000 => call_addr(self),

            // 3XNN
            0x3000 => se_vx_byte(self),

            // 4XNN
            0x4000 => sne_vx_byte(self),

            // 5XY0
            0x5000 => se_vx_vy(self),

            // 6XNN
            0x6000 => ld_vx_byte(self),

            // 7XNN
            0x7000 => add_vx_byte(self),

            0x8000 => match self.opcode & 0x000F {

                // 8XY0
                0x0000 => ld_vx_vy(self),

                // 8XY1
                0x0001 => or_vx_vy(self),

                // 8XY2
                0x0002 => and_vx_vy(self),

                // 8XY3
                0x0003 => xor_vx_vy(self),

                // 8XY4
                0x0004 => add_vx_vy(self),

                // 8XY5
                0x0005 => sub_vx_vy(self),

                // 8XY6
                0x0006 => shr_vx_vy(self),

                // 8XY7
                0x0007 => subn_vx_vy(self),

                // 8XYE
                0x000E => shl_vx_vy(self),

                _ => println!("Unknown opcode [0x8000]: {:02X}", self.opcode)
            },

            // 9XY0
            0x9000 => sne_vx_vy(self),

            // ANNN
            0xA000 => ld_i_addr(self),

            // BNNN
            0xB000 => jp_v0_addr(self),

            // CXNN
            0xC000 => rnd_vx_byte(self),

            // DXYN
            0xD000 => drw_vx_vy_n(self),

            0xE000 => match self.opcode & 0x00FF {

                // EX9E
                0x009E => skp_vx(self),

                // EXA1
                0x00A1 => sknp_vx(self),

                _ => println!("Unknown opcode [0xE000]: {:02X}", self.opcode)
            },

            0xF000 => match self.opcode & 0x00FF {

                // FX07
                0x0007 => ld_vx_dt(self),

                // FX0A
                0x000A => ld_vx_k(self),

                // FX15
                0x0015 => ld_dt_vx(self),

                // FX18
                0x0018 => ld_st_vx(self),

                // FX1E
                0x001E => add_i_vx(self),

                // FX29
                0x0029 => ld_f_vx(self),

                // FX33
                0x0033 => ld_b_vx(self),

                // FX55
                0x0055 => ld_i_vx(self),

                // FX65
                0x0065 => ld_vx_i(self),

                _ => println!("Unknown opcode [0xF000]: {:02X}", self.opcode)
            },

            _ => println!("Unknown opcode: {:02X}", self.opcode),
        };
    }

    pub fn update_timers(&mut self) {

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

    pub fn emulate_cycle(&mut self) {
        // fetch opcode: merge two memory locations for an opcode (build opcode with next two bytes)
        self.opcode = (self.ram[self.pc as usize] as u16) << 8 | self.ram[self.pc as usize + 1] as u16;

        println!("Executing opcode 0x{:X}", self.opcode);

        self.translate_opcode();

        self.update_timers();
    }
}