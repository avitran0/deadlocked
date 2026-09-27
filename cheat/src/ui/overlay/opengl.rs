use egui_glow::glow::{self, HasContext as _};

pub fn build_program(
    glow: &glow::Context,
    vertex_source: &str,
    fragment_source: &str,
) -> Result<glow::Program, String> {
    let program = unsafe { glow.create_program() }.map_err(|e| e.to_string())?;
    let vertex = compile_shader(glow, glow::VERTEX_SHADER, vertex_source)?;
    let fragment = compile_shader(glow, glow::FRAGMENT_SHADER, fragment_source)?;

    unsafe {
        glow.attach_shader(program, vertex);
        glow.attach_shader(program, fragment);
        glow.link_program(program);
    }

    if !unsafe { glow.get_program_link_status(program) } {
        let log = unsafe { glow.get_program_info_log(program) };
        unsafe {
            glow.delete_shader(vertex);
            glow.delete_shader(fragment);
            glow.delete_program(program);
        }
        return Err(log);
    }

    unsafe {
        glow.delete_shader(vertex);
        glow.delete_shader(fragment);
    }

    Ok(program)
}

pub fn uniform_location(
    glow: &glow::Context,
    program: glow::Program,
    name: &str,
) -> Result<glow::UniformLocation, String> {
    unsafe { glow.get_uniform_location(program, name) }
        .ok_or_else(|| format!("missing uniform {name}"))
}

fn compile_shader(
    glow: &glow::Context,
    shader_type: u32,
    source: &str,
) -> Result<glow::Shader, String> {
    let shader = unsafe { glow.create_shader(shader_type) }.map_err(|e| e.to_string())?;
    unsafe {
        glow.shader_source(shader, source);
        glow.compile_shader(shader);
    }

    if !unsafe { glow.get_shader_compile_status(shader) } {
        let log = unsafe { glow.get_shader_info_log(shader) };
        unsafe { glow.delete_shader(shader) };
        return Err(log);
    }

    Ok(shader)
}
