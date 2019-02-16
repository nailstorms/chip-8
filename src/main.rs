#[macro_use]
extern crate glium;
extern crate rand;

use glium::{Display, Surface, glutin};
use std::env;

mod chip8;


fn main() {

    ///////////////////// COMMAND LINE ARGS /////////////////////
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: chip8 [PATH_TO_ROM]");
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
    let vertex1 = Vertex { position: [-0.5, 0.5], tex_coords: [0.0, 1.0]};
    let vertex2 = Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 1.0]};
    let vertex3 = Vertex { position: [ 0.5, -0.5], tex_coords: [1.0, 0.0]};
    let vertex4 = Vertex { position: [ -0.5, -0.5], tex_coords: [0.0, 0.0]};
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
    let mut chip8_vm = chip8::Vm::new();

    // load the game into the memory
    chip8_vm.load_game(path_to_game);

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
        chip8_vm.set_keys();
    }
}