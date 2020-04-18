use super::gl;

pub fn program_from_shaders( vtx_shader : gl::GLuint, frag_shader : gl::GLuint, error_dest : &mut [i8]  ) -> Option<gl::GLuint> {
    let program_id;
    let mut success: gl::GLint = 1;
    unsafe {
        program_id = gl::CreateProgram();
        gl::AttachShader(program_id, vtx_shader );
        gl::AttachShader(program_id, frag_shader );
        gl::LinkProgram(program_id);
        gl::DetachShader(program_id, vtx_shader );
        gl::DetachShader(program_id, frag_shader );

        gl::GetProgramiv(program_id,  gl::LINK_STATUS, &mut success);
        if success == 0 {
            gl::GetProgramInfoLog( program_id, error_dest.len() as i32,  0 as *mut _, error_dest.as_mut_ptr() as *mut u8 );
            return None;
        }
    }
    return Some( program_id );
}

pub fn shader_from_source( shader_source : *const u8, kind: gl::GLenum, error_dest : &mut [i8] ) -> Option<gl::GLuint> {
    let id;
    let mut success: gl::GLint = 1;
    unsafe {
        id = gl::CreateShader(kind);
        gl::ShaderSource(id, 1, &shader_source, 0 as *const _);
        gl::CompileShader(id);
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        unsafe{ gl::GetShaderInfoLog( id, error_dest.len() as i32,  0 as *mut _, error_dest.as_mut_ptr() as *mut gl::GLchar ); }
        return None;
    }
    return Some( id );
}