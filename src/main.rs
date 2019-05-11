#[macro_use]
extern crate glium;
extern crate rand;

use glium::{Display, Surface, glutin};
use std::env;

mod vm;


fn main() {

    ///////////////////// COMMAND LINE ARGS /////////////////////
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: chip-8 [PATH_TO_ROM]");
        return;
    }
    let path_to_game = &args[1];

    ///////////////////// CREATE DISPLAY /////////////////////

    let mut events_loop = glutin::EventsLoop::new();
    let context = glutin::ContextBuilder::new();
    let window = glutin::WindowBuilder::new()
        .with_title(format!("сhip-8"));

    let display = Display::new(window, context, &events_loop).unwrap();


    ///////////////////// SETUP VERTEXES /////////////////////

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
        tex_coords: [f32; 2], // this is new
    }
    implement_vertex!(Vertex, position, tex_coords);
    let vertex1 = Vertex { position: [-0.5, 0.5], tex_coords: [0.0, 0.0]};
    let vertex2 = Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 0.0]};
    let vertex3 = Vertex { position: [ 0.5, -0.5], tex_coords: [1.0, 1.0]};
    let vertex4 = Vertex { position: [ -0.5, -0.5], tex_coords: [0.0, 1.0]};
    let shape = vec![vertex1, vertex2, vertex3, vertex4];

    // uploading this shape to the memory of our video card in what is called a vertex buffer
    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();

    // to tell OpenGL how to link these vertices together to obtain triangles.
    let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3]).unwrap();

    /////////////////// SHADERS ///////////////////
    let vertex_shader_src = r#"
	    #version 140
	    in vec2 position;
	    in vec2 tex_coords;
	    out vec2 v_tex_coords;
	    void main() {
	    	v_tex_coords = tex_coords; // just pass the texture coordinates through
	    	gl_Position = vec4(position, 0.0, 1.0);
	    }
		"#;

    let fragment_shader_src = r#"
	    #version 140
	    in vec2 v_tex_coords;
	    out vec4 color;
	    uniform sampler2D tex;
	    void main() {
	    	// texture() is an openGL method.
	        color = texture(tex, v_tex_coords);
	    }
		"#;

    // send shaders source code to the glium library
    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();



    //////////// INITIALIZE CHIP 8 SYSTEM //////////////

    // initialize the chip8 system
    let mut chip8_vm = vm::Vm::init();

    // load the game into the memory
    chip8_vm.load_game(path_to_game);

    ///////////////////////////////////////////////////

    ///////////////// KEY MAPPING /////////////////////

//    fn map_keys(k: glutin::KeyboardInput) -> Option<u8> {
//        if let Some() = k.virtual_keycode {
//            return match k {
//                glutin::VirtualKeyCode::Key1 => Some(0x1),
//                glutin::VirtualKeyCode::Key2 => Some(0x2),
//                glutin::VirtualKeyCode::Key3 => Some(0x3),
//
//                glutin::VirtualKeyCode::Q => Some(0x4),
//                glutin::VirtualKeyCode::W => Some(0x5),
//                glutin::VirtualKeyCode::E => Some(0x6),
//
//                glutin::VirtualKeyCode::A => Some(0x7),
//                glutin::VirtualKeyCode::S => Some(0x8),
//                glutin::VirtualKeyCode::D => Some(0x9),
//
//                glutin::VirtualKeyCode::Z => Some(0xA),
//                glutin::VirtualKeyCode::X => Some(0x0),
//                glutin::VirtualKeyCode::C => Some(0xB),
//
//                glutin::VirtualKeyCode::Key4 => Some(0xC),
//                glutin::VirtualKeyCode::R => Some(0xD),
//                glutin::VirtualKeyCode::F => Some(0xE),
//                glutin::VirtualKeyCode::V => Some(0xF),
//
//                _ => None
//            }
//        }
//        return None;
//    }

    ///////////////////////////////////////////////////


    // emulation loop
    let mut closed = false;
    while !closed {
        // emulate one cycle
        chip8_vm.emulate_cycle();

        // if the draw flag is set, update the screen
        if chip8_vm.draw_flag {

            /////////////  UPDATE TEXTURE ///////////////
            let mut image = vec![vec![(0.0, 0.0, 0.0); 64]; 32];

            for row in 0..32 {
                for col in 0..64 {
                    if chip8_vm.screen[row * 64 + col]  == 1 {
                        image[row][col] = (1.0, 1.0, 1.0);
                    } else {
                        image[row][col] = (0.0, 0.0, 0.0);
                    }
                }
            }

            let texture = glium::texture::Texture2d::new(&display, image).unwrap();
            ////////////////////////////////////////////////

            ///////////////////// DRAW /////////////////////
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 1.0, 1.0);

            let uniforms = uniform! {
				tex: texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
			};
            target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
            target.finish().unwrap();

            ///////////////////////////////////////////////

            events_loop.poll_events(|ev| {
                match ev {
                    glutin::Event::WindowEvent { event, .. } => match event {
                        glutin::WindowEvent::CloseRequested => closed = true,
                        _ => (),
                    },
                    _ => (),
                }
            });
        }

        // store key press stage (Press and Release)
        // chip8_vm.set_keys();
    }
}