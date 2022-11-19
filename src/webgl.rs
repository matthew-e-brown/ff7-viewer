use std::fmt::Display;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlShader};


/// A type of shader that can be compiled.
#[derive(Clone, Copy)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl From<ShaderType> for u32 {
    fn from(shader_type: ShaderType) -> Self {
        match shader_type {
            ShaderType::Vertex => WebGl2RenderingContext::VERTEX_SHADER,
            ShaderType::Fragment => WebGl2RenderingContext::FRAGMENT_SHADER,
        }
    }
}

impl Display for ShaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderType::Vertex => f.write_str("vertex"),
            ShaderType::Fragment => f.write_str("fragment"),
        }
    }
}


/// Grabs the a canvas from the page, gets its WebGL 2 context, and sets the canvas's size based on the screens display
/// ratio. Panics if any of the above fail, as they are unrecoverable errors.
pub fn init_viewport(canvas_id: &str) -> (HtmlCanvasElement, WebGl2RenderingContext) {
    /// Used to convert multiplied canvas sizes back to unsigned numbers
    fn f64_to_u32(n: f64) -> u32 {
        match n {
            _ if n <= u32::MIN as f64 => u32::MIN,
            _ if n >= u32::MAX as f64 => u32::MAX,
            _ => n as u32,
        }
    }

    let window = web_sys::window().expect("Global `window` object is missing.");
    let document = window.document().expect("`window` is missing `document`.");

    // Get the canvas as an `Element`
    let canvas = document
        .get_element_by_id(canvas_id)
        .expect(&format!("Could not find canvas with ID {canvas_id}."));

    // Cast it into an `HtmlCanvasElement`
    let canvas: HtmlCanvasElement = canvas
        .dyn_into()
        .expect(&format!("Element #{canvas_id} is not a <canvas> element."));

    let gl: WebGl2RenderingContext = canvas
        .get_context("webgl2")
        .expect("WebGL 2 context is not available.")
        .expect("WebGL 2 context is not available.")
        .dyn_into()
        .expect("WebGL 2 context is not available.");

    // Increase the resolution based on the device's pixel ratio
    let dpi = window.device_pixel_ratio();
    let css = canvas.style();

    let src_w: f64 = canvas.width().into();
    let src_h: f64 = canvas.height().into();

    let new_w = f64_to_u32((src_w * dpi).round());
    let new_h = f64_to_u32((src_h * dpi).round());

    canvas.set_width(new_w);
    canvas.set_height(new_h);

    css.set_property("width", &format!("{:.5}px", src_w * dpi))
        .expect("Could not set canvas width.");
    css.set_property("height", &format!("{:.5}px", src_h * dpi))
        .expect("Could not set canvas height.");

    (canvas, gl)
}


/// Turns a vertex and fragment shader pair into a WebGL program.
pub fn init_program(gl: &WebGl2RenderingContext, vert_input: &str, frag_input: &str) -> Result<WebGlProgram, JsValue> {
    let vert_input = vert_input.trim();
    let frag_input = frag_input.trim();

    let vert_shader = init_shader(gl, ShaderType::Vertex, vert_input)?;
    let frag_shader = init_shader(gl, ShaderType::Fragment, frag_input)?;

    let program = gl
        .create_program()
        .ok_or::<JsValue>("Could not create WebGL program.".into())?;

    gl.attach_shader(&program, &vert_shader);
    gl.attach_shader(&program, &frag_shader);
    gl.link_program(&program);

    let status = gl.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS);
    let status = status.as_bool().or(Some(false)).unwrap();

    if status {
        Ok(program)
    } else {
        let log_info = gl
            .get_program_info_log(&program)
            .or_else(|| Some("No logs found.".to_owned()))
            .unwrap();
        let vert_info = gl
            .get_shader_info_log(&vert_shader)
            .or_else(|| Some("No logs found.".to_owned()))
            .unwrap();
        let frag_info = gl
            .get_shader_info_log(&frag_shader)
            .or_else(|| Some("No logs found.".to_owned()))
            .unwrap();

        gl.delete_program(Some(&program));
        gl.delete_shader(Some(&vert_shader));
        gl.delete_shader(Some(&frag_shader));

        let msg = format!(
            "Could not initialize program:\n{}\n\tVert shader log:\n{}\n\tFrag shader log:\n{}",
            log_info, vert_info, frag_info
        );

        Err(msg.into())
    }
}


/// Compiles a WebGL shader.
fn init_shader(gl: &WebGl2RenderingContext, shader_type: ShaderType, source: &str) -> Result<WebGlShader, JsValue> {
    let shader = gl
        .create_shader(shader_type.into())
        .ok_or::<JsValue>(format!("Could not create {shader_type} shader.").into())?;

    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    Ok(shader)
}
