extern crate rand;
extern crate sdl2;

mod vm;
mod ui;
mod opcodes;

use std::env;
use std::time::Duration;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use crate::vm::Vm;
use crate::ui::Ui;

static SCALE: u32 = 12;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let args: Vec<String> = env::args().collect();
    let mut vm = Vm::init();
    vm.load_font();

    vm.load_game(&args[1]);

    let mut ui = Ui::init(&sdl_context, SCALE);

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(keycode), .. } => ui.set_key_pressed(&mut vm, keycode),
                Event::KeyUp { keycode: Some(keycode), .. } => ui.set_key_released(&mut vm, keycode),
                _ => {}
            }
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 600));
        vm.emulate_cycle(&mut ui);
        if vm.draw_flag {
            ui.draw_canvas(&mut vm, SCALE);
        }
    }
}