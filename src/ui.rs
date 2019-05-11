use crate::vm::Vm;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::keyboard::Keycode;
use sdl2::Sdl;

pub struct Ui {
    pub canvas: Canvas<Window>
}

impl Ui {

    /// Creates a new `Ui` instance with default state
    pub fn init(sdl_context: &Sdl, scale: u32) -> Ui {

        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("chip-8", 64 * scale, 32 * scale)
            .position_centered()
            .opengl()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Ui {
            canvas: canvas,
        }
    }

    pub fn set_key_pressed(&mut self, vm: &mut Vm, keycode: Keycode) {
        match keycode {
            Keycode::Num1 => { vm.key_states[0x1] = true; },
            Keycode::Num2 => { vm.key_states[0x2] = true; },
            Keycode::Num3 => { vm.key_states[0x3] = true; },
            Keycode::Q => { vm.key_states[0x4] = true; },
            Keycode::W => { vm.key_states[0x5] = true; },
            Keycode::E => { vm.key_states[0x6] = true; },
            Keycode::A => { vm.key_states[0x7] = true; },
            Keycode::S => { vm.key_states[0x8] = true; },
            Keycode::D => { vm.key_states[0x9] = true; },
            Keycode::Z => { vm.key_states[0xA] = true; },
            Keycode::X => { vm.key_states[0x0] = true; },
            Keycode::C => { vm.key_states[0xB] = true; },
            Keycode::Num4 => { vm.key_states[0xC] = true; },
            Keycode::R => { vm.key_states[0xD] = true; },
            Keycode::F => { vm.key_states[0xE] = true; },
            Keycode::V => { vm.key_states[0xF] = true; },
            _ => {}
        }
    }

    pub fn set_key_released(&mut self, vm: &mut Vm, keycode: Keycode) {
        match keycode {
            Keycode::Num1 => { vm.key_states[0x1] = false; },
            Keycode::Num2 => { vm.key_states[0x2] = false; },
            Keycode::Num3 => { vm.key_states[0x3] = false; },
            Keycode::Q => { vm.key_states[0x4] = false; },
            Keycode::W => { vm.key_states[0x5] = false; },
            Keycode::E => { vm.key_states[0x6] = false; },
            Keycode::A => { vm.key_states[0x7] = false; },
            Keycode::S => { vm.key_states[0x8] = false; },
            Keycode::D => { vm.key_states[0x9] = false; },
            Keycode::Z => { vm.key_states[0xA] = false; },
            Keycode::X => { vm.key_states[0x0] = false; },
            Keycode::C => { vm.key_states[0xB] = false; },
            Keycode::Num4 => { vm.key_states[0xC] = false; },
            Keycode::R => { vm.key_states[0xD] = false; },
            Keycode::F => { vm.key_states[0xE] = false; },
            Keycode::V => { vm.key_states[0xF] = false; },
            _ => {}
        }
    }

    /// Draws the CPU's display to the canvas
    pub fn draw_canvas(&mut self, vm: &mut Vm, scale: u32) {
        for i in 0..64 * 32 {
            let current_pixel = vm.screen[i];
            let x = (i % 64) * scale as usize;
            let y = (i / 64) * scale as usize;

            self.canvas.set_draw_color(Color::RGB(0, 0, 0));
            if current_pixel == 1 {
                self.canvas.set_draw_color(Color::RGB(255, 255, 255));
            }
            let _ = self.canvas.fill_rect(Rect::new(x as i32, y as i32, scale, scale));
        }
        self.canvas.present();
    }

}