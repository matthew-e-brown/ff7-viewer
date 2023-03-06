#![allow(dead_code)] // Temporary

use gl::types::*;
use glfw::WindowMode::Windowed;
use glfw::{Action, Context, Key, Window, WindowEvent};


pub trait ToBuffer {}


#[allow(dead_code)]
struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
}


const VERT_SHADER_SOURCE: &str = include_str!("./shaders/vert.glsl");
const FRAG_SHADER_SOURCE: &str = include_str!("./shaders/frag.glsl");

const VERTEX_COUNT: usize = 3;
const VERTICES: [Vertex; VERTEX_COUNT] = [
    Vertex {
        x: -0.5,
        y: -0.5,
        z: 0.0,
        r: 1.0,
        g: 0.0,
        b: 0.0,
    },
    Vertex {
        x: 0.5,
        y: -0.5,
        z: 0.0,
        r: 0.0,
        g: 1.0,
        b: 0.0,
    },
    Vertex {
        x: 0.0,
        y: 0.5,
        z: 0.0,
        r: 0.0,
        g: 0.0,
        b: 1.0,
    },
];


pub fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // Request OpenGL version 4.6
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(glfw::WindowHint::DoubleBuffer(true));
    glfw.window_hint(glfw::WindowHint::FocusOnShow(true));
    glfw.window_hint(glfw::WindowHint::Focused(true));

    let (mut window, events) = glfw
        .create_window(512, 512, "Hello, GLFW!", Windowed)
        .expect("Could not create an OpenGL 4.6 window.");

    // Pass OpenGL load calls to GLFW
    gl::load_with(|s| window.get_proc_address(s));

    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    window.set_resizable(false);
    window.set_key_polling(true);
    window.make_current();

    let (width, height) = window.get_framebuffer_size();
    unsafe { gl::Viewport(0, 0, width, height) };

    // Mutable because CreateBuffers will change these to the proper values
    let mut vbo: GLuint = 0;

    {
        let size_of = std::mem::size_of::<[Vertex; VERTEX_COUNT]>()
            .try_into()
            .expect("Vertex data is too large.");
        let pointer = VERTICES.as_ptr().cast();
        unsafe {
            // glCreateBuffers actually expects an array, but since an "array" is just a pointer, we just pass the
            // single reference.
            gl::CreateBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, size_of, pointer, gl::STATIC_DRAW);
        }
    }

    let vert_shader = unsafe { compile_shader(gl::VERTEX_SHADER, VERT_SHADER_SOURCE) }.unwrap();
    let frag_shader = unsafe { compile_shader(gl::FRAGMENT_SHADER, FRAG_SHADER_SOURCE) }.unwrap();

    let program = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(program, vert_shader);
        gl::AttachShader(program, frag_shader);
        gl::LinkProgram(program);
    }

    // Error check program
    unsafe {
        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if (success as GLboolean) == gl::FALSE {
            let mut log_size = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_size);

            let mut buffer = vec![0; log_size as usize];
            gl::GetProgramInfoLog(program, log_size, std::ptr::null_mut(), buffer.as_mut_ptr().cast());

            let log_output = String::from_utf8_lossy(&buffer);
            panic!("{}", log_output.into_owned());
        }
    }

    unsafe {
        gl::DeleteShader(vert_shader);
        gl::DeleteShader(frag_shader);
    }

    let mut vao: GLuint = 0;
    unsafe {
        gl::CreateVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }

    unsafe {
        let v_size: i32 = std::mem::size_of::<Vertex>().try_into().unwrap();
        let f_size: i32 = std::mem::size_of::<f32>().try_into().unwrap();
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, v_size, (f_size * 0) as *const _);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, v_size, (f_size * 3) as *const _);
        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);
    }

    while !window.should_close() {
        unsafe {
            gl::ClearColor(0.17, 0.17, 0.17, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(program);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, VERTEX_COUNT as i32);
        }

        window.swap_buffers();
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }
    }
}


unsafe fn compile_shader(shader_type: GLuint, source: &str) -> Result<GLuint, String> {
    let shader = gl::CreateShader(shader_type);
    let src = source.as_bytes().as_ptr().cast::<i8>();
    let len: i32 = source.len().try_into().or(Err("Shader source is too long.".to_owned()))?;

    // glShaderSource *actually* expects two arrays here, but since they expect C-style arrays and we've told them that
    // there'll be only one, we can just pass the pointers directly.
    gl::ShaderSource(shader, 1, &src, &len);
    gl::CompileShader(shader);

    let mut success = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

    if (success as GLboolean) == gl::FALSE {
        let mut log_size = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_size);

        let mut buffer = vec![0; log_size as usize];
        gl::GetShaderInfoLog(shader, log_size, std::ptr::null_mut(), buffer.as_mut_ptr().cast());

        let log_output = String::from_utf8_lossy(&buffer[..]);
        println!("Could not compile shader. Info log:\n{}", log_output);

        gl::DeleteShader(shader);
        Err(log_output.into_owned())
    } else {
        Ok(shader)
    }
}


fn handle_window_event(window: &mut Window, event: WindowEvent) {
    match event {
        WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true);
        },
        _ => (),
    }
}
