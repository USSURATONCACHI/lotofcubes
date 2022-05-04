use gl;
use std;
use std::ffi::{CString, CStr};
use std::path::PathBuf;

use crate::{mat, resources};
use resources::Resources;
use crate::glsl_expand::ExpandError;
use crate::rgl::Error::{CompileError, CStringError, GlslExpandError, LinkError, UnknownShaderType};

#[derive(Debug)]
pub enum Error {
    UnknownShaderType { name: String },
    ResourceLoad { name: String, inner: resources::Error },
    CompileError { name: String, message: String },
    LinkError { name: String, message: String },

    CStringError { error: std::ffi::NulError },
    GlslExpandError { error: ExpandError },
}
pub struct Shader {
    id: gl::types::GLuint,
}
impl Shader {
    pub fn from_res(res: &mut Resources, name: &str) -> Result<Shader, Error> {
        const POSSIBLE_EXT: [(&str, gl::types::GLenum); 2] = [
            (".vert", gl::VERTEX_SHADER),
            (".frag", gl::FRAGMENT_SHADER),
        ];

        let shader_kind = POSSIBLE_EXT.iter()
            .find(|&&(file_extension, _)| {
                name.ends_with(file_extension)
            })
            .map(|&(_, kind)| kind)
            .ok_or_else(|| UnknownShaderType { name: name.into() })?;

        /*let source = res.load_cstring(name)
            .map_err(|e| ResourceLoad { name: name.into(), inner: e })?;*/
        let text = res.load_shader_text(PathBuf::from(name)).map_err(|e| GlslExpandError { error: e })?;
        let source = std::ffi::CString::new(&text[..]).map_err(|e| CStringError { error: e })?;

        Shader::from_source(&source, shader_kind)
            .map_err(|msg| CompileError { name: name.into(), message: msg } )
    }

    pub fn from_source(source: &CStr, shader_type: gl::types::GLenum) -> Result<Shader, String> {
        let id = shader_from_source(source, shader_type)?;
        Ok(Shader{id})
    }
    pub fn from_vert_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::FRAGMENT_SHADER)
    }

    pub fn id(&self) -> gl::types::GLuint {
    	self.id
    }
}
impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id); }
    }
}

fn shader_from_source(source: &CStr, shader_type: gl::types::GLenum) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl::CreateShader(shader_type) };

    //Проверка на успешную компиляцию
    let mut success: gl::types::GLint = 1;
    unsafe {
        gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(id);
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        //Получение длины текста ошибки и самого текста
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error: CString = create_whitespace_cstring(len as usize);

        unsafe {
            gl::GetShaderInfoLog(id, len, std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar);
        }
        return Err(error.to_string_lossy().into_owned());
    } 

    Ok(id)
}

fn create_whitespace_cstring(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    unsafe { CString::from_vec_unchecked(buffer) }
}


pub struct Program {
    id: gl::types::GLuint,
    locations: Vec<i32>,
}
impl Program {
    pub fn from_res(res: &mut Resources, name: &str, uniforms: Vec<&str>) -> Result<Program, Error> {
        const POSSIBLE_EXT: [&str; 2] = [
            ".vert",
            ".frag",
        ];

        let shaders = POSSIBLE_EXT.iter()
            .map(|file_extension| {
                Shader::from_res(res, &format!("{}{}", name, file_extension))
            })
            .collect::<Result<Vec<Shader>, Error>>()?;

        Program::from_shaders(&shaders[..], uniforms).map_err(|msg| LinkError { name: name.into(), message: msg })
    }

	pub fn from_shaders(shaders: &[Shader], uniforms: Vec<&str>) -> Result<Program, String> {
		let program_id = unsafe { gl::CreateProgram() };

		for s in shaders {
			unsafe { gl::AttachShader(program_id, s.id()) };
		}

		unsafe { gl::LinkProgram(program_id) };
		let mut success: gl::types::GLint = 1;
		unsafe {
		    gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
		}

		if success == 0 {
		    let mut len: gl::types::GLint = 0;
		    unsafe {
		        gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
		    }

		    let error = create_whitespace_cstring(len as usize);

		    unsafe {
		        gl::GetProgramInfoLog(
		            program_id,
		            len,
		            std::ptr::null_mut(),
		            error.as_ptr() as *mut gl::types::GLchar
		        );
		    }

		    return Err(error.to_string_lossy().into_owned());
		}

		for s in shaders {
			unsafe { gl::DetachShader(program_id, s.id()) };
		}

        unsafe { gl::UseProgram(program_id); }
        let mut locations: Vec<i32> = vec![];
        for name in uniforms {
            locations.push(uniform_loc(program_id, name));
        }
        Ok(Program { id: program_id, locations })
	}

    pub fn load_locations(&mut self, uniforms: Vec<&str>) {
        self.set_used();
        for name in uniforms {
            self.locations.push(uniform_loc(self.id, name));
        }
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
    pub fn set_used(&self) {
    	unsafe { gl::UseProgram(self.id); };
    }

    /** uniform_id идут в том порядке, в котором были загружены (uniforms: Vec<&str>). Это локальный id, а не id opengl*/
    pub fn uniform_matrix4fv(&self, uniform_id: usize, transpose: bool, ptr: *const gl::types::GLfloat) {
        unsafe { gl::UniformMatrix4fv(self.locations[uniform_id], 1, if transpose {1u8} else {0u8}, ptr); }
    }
    pub fn uniform_mat4(&self, uniform_id: usize, mat: &mat::Mat4) {
        unsafe { gl::UniformMatrix4fv(self.locations[uniform_id], 1, 1, mat.0.as_ptr()); }
    }
    pub fn uniform3f   (&self, uniform_id: usize, a: f32, b: f32, c: f32) {
        unsafe { gl::Uniform3f(self.locations[uniform_id], a, b, c); }
    }
    pub fn uniform1f   (&self, uniform_id: usize, a: f32) {
        unsafe { gl::Uniform1f(self.locations[uniform_id], a); }
    }
    pub fn uniform2f   (&self, uniform_id: usize, x: f32, y: f32) {
        unsafe { gl::Uniform2f(self.locations[uniform_id], x, y); }
    }
    pub fn uniform1ui  (&self, uniform_id: usize, a: u32) {
        unsafe { gl::Uniform1ui(self.locations[uniform_id], a); }
    }
    pub fn uniform1i   (&self, uniform_id: usize, i: i32) {
        unsafe { gl::Uniform1i(self.locations[uniform_id], i); }
    }


    pub fn uniform1uiv (&self, uniform_id: usize, data: &Vec<u32>) {
        unsafe { gl::Uniform1uiv(self.locations[uniform_id], data.len() as i32, data.as_ptr() as *const u32); }
    }
    pub fn uniform1iv (&self, uniform_id: usize, data: &Vec<i32>) {
        unsafe { gl::Uniform1iv(self.locations[uniform_id], data.len() as i32, data.as_ptr()); }
    }


    pub fn locations(&self) -> &Vec<i32> { &self.locations }
}
impl Drop for Program {
	fn drop(&mut self) {
		unsafe { gl::DeleteProgram(self.id); }
	}
}

fn uniform_loc(program_id: u32, name: &str) -> i32 {
    unsafe {
        let name = std::ffi::CString::new(name).unwrap();
        return gl::GetUniformLocation(program_id, name.as_ptr() as *const i8)
    }
}


pub struct Model {
    pub vao: gl::types::GLuint,
    pub indices_count: i32,

    vbo: gl::types::GLuint,
    ebo: gl::types::GLuint,
}
impl Model {
    pub fn create(vao: gl::types::GLuint, vbo: gl::types::GLuint, ebo: gl::types::GLuint, indices_count: i32) -> Model {
        Model { vao, vbo, ebo, indices_count }
    }
    pub fn render(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices_count,
                gl::UNSIGNED_INT,
                std::ptr::null() as *const gl::types::GLvoid
            );
        }
    }
}
impl Model {
    pub fn cube() -> Model {
        let vertices: Vec<f32> = vec![
            -0.5, -0.5, -0.5, 1.0, 0.0, 0.0,
            0.5, -0.5, -0.5, 0.0, 1.0, 0.0,
            0.5,  0.5, -0.5, 1.0, 1.0, 0.0,
            -0.5,  0.5, -0.5, 0.0, 0.0, 1.0,
            -0.5, -0.5,  0.5, 1.0, 0.0, 1.0,
            0.5, -0.5,  0.5, 0.0, 1.0, 1.0,
            0.5,  0.5,  0.5, 1.0, 1.0, 1.0,
            -0.5,  0.5,  0.5, 0.0, 0.0, 0.0,
        ];
        let indices: Vec<u32> = vec![
            0, 1, 2, 2, 3, 0,
            0, 4, 5, 0, 5, 1,
            0, 3, 4, 3, 7, 4,
            3, 2, 6, 6, 7, 3,
            4, 7, 6, 6, 5, 4,
            1, 5, 6, 6, 2, 1,
        ];
        let mut vbo: gl::types::GLuint = 0;
        let mut vao: gl::types::GLuint = 0;
        let mut ebo: gl::types::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER, // target
                (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
                vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );
            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl::VertexAttribPointer(
                0, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null() // offset of the first component
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid // offset of the first component
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER, // target
                (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr, // size of data in bytes
                indices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0); // unbind the buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
        Model { vao, vbo, ebo, indices_count: indices.len() as i32 }
    }

    pub fn fractal(iter: usize) -> Model {
        let side_size = 3i32.pow(iter as u32 - 1);
        let mut grid: Vec<bool> = [false; 1].into_iter().cycle().take(side_size.pow(3) as usize).collect();
        build_f_cube(&mut grid, side_size, 0, 0, 0, iter as i32);

        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let shift = side_size as f32 / 2.0;

        for x in 0_i32..side_size {
            for y in 0_i32..side_size {
                for z in 0_i32..side_size {
                    if !grid[ (z * side_size.pow(2) + y * side_size + x) as usize] { continue; }
                    let r = x as f32 / (side_size as f32);
                    let g = y as f32 / (side_size as f32);
                    let b = z as f32 / (side_size as f32);

                    if (z - 1) < 0 || !grid[ ((z - 1) * side_size.pow(2) + y * side_size + x) as usize] {
                        let start_index = (vertices.len()  / 6) as u32;
                        add_vertex(&mut vertices, x as f32,         y as f32,           z as f32, 0.0, 0.0, b, shift);
                        add_vertex(&mut vertices, x as f32 + 1.0,   y as f32,           z as f32, 0.0, 0.0, b, shift);
                        add_vertex(&mut vertices, x as f32 + 1.0,   y as f32 + 1.0,     z as f32, 0.0, 0.0, b, shift);
                        add_vertex(&mut vertices, x as f32,         y as f32 + 1.0,     z as f32, 0.0, 0.0, b, shift);
                        add_index(&mut indices, 0, 1, 2, start_index);
                        add_index(&mut indices, 2, 3, 0, start_index);
                    }
                    if (z + 1) >= side_size || !grid[ ((z + 1) * side_size.pow(2) + y * side_size + x) as usize] {
                        let start_index = (vertices.len() / 6) as u32;
                        add_vertex(&mut vertices, x as f32,         y as f32,       z as f32 + 1.0, 0.0, 0.0, b, shift);
                        add_vertex(&mut vertices, x as f32 + 1.0,   y as f32,       z as f32 + 1.0, 0.0, 0.0, b, shift);
                        add_vertex(&mut vertices, x as f32 + 1.0,   y as f32 + 1.0, z as f32 + 1.0, 0.0, 0.0, b, shift);
                        add_vertex(&mut vertices, x as f32,         y as f32 + 1.0, z as f32 + 1.0, 0.0, 0.0, b, shift);
                        add_index(&mut indices, 0, 3, 2, start_index);
                        add_index(&mut indices, 2, 1, 0, start_index);
                    }
                    if (y - 1) < 0 || !grid[ (z * side_size.pow(2) + (y - 1) * side_size + x) as usize] {
                        let start_index = (vertices.len()  / 6) as u32;
                        add_vertex(&mut vertices, x as f32,         y as f32,     z as f32,       0.0, g, 0.0, shift);
                        add_vertex(&mut vertices, x as f32 + 1.0,   y as f32,     z as f32,       0.0, g, 0.0, shift);
                        add_vertex(&mut vertices, x as f32 + 1.0,   y as f32,     z as f32 + 1.0, 0.0, g, 0.0, shift);
                        add_vertex(&mut vertices, x as f32,         y as f32,     z as f32 + 1.0, 0.0, g, 0.0, shift);
                        add_index(&mut indices, 0, 3, 2, start_index);
                        add_index(&mut indices, 2, 1, 0, start_index);
                    }
                    if (y + 1) >= side_size || !grid[ (z * side_size.pow(2) + (y + 1) * side_size + x) as usize] {
                        let start_index = (vertices.len() / 6) as u32;
                        add_vertex(&mut vertices, x as f32,         y as f32 + 1.0, z as f32,       0.0, g, 0.0, shift);
                        add_vertex(&mut vertices, x as f32 + 1.0,   y as f32 + 1.0, z as f32,       0.0, g, 0.0, shift);
                        add_vertex(&mut vertices, x as f32 + 1.0,   y as f32 + 1.0, z as f32 + 1.0, 0.0, g, 0.0, shift);
                        add_vertex(&mut vertices, x as f32,         y as f32 + 1.0, z as f32 + 1.0, 0.0, g, 0.0, shift);
                        add_index(&mut indices, 0, 1, 2, start_index);
                        add_index(&mut indices, 2, 3, 0, start_index);
                    }//

                    if (x - 1) < 0 || !grid[ (z * side_size.pow(2) + y * side_size + x - 1) as usize] {
                        let start_index = (vertices.len()  / 6) as u32;
                        add_vertex(&mut vertices, x as f32,   y as f32,         z as f32, r, 0.0, 0.0, shift);
                        add_vertex(&mut vertices, x as f32,   y as f32 + 1.0,     z as f32, r, 0.0, 0.0, shift);
                        add_vertex(&mut vertices, x as f32,   y as f32 + 1.0,     z as f32 + 1.0, r, 0.0, 0.0, shift);
                        add_vertex(&mut vertices, x as f32,   y as f32,         z as f32 + 1.0, r, 0.0, 0.0, shift);
                        add_index(&mut indices, 0, 1, 2, start_index);
                        add_index(&mut indices, 2, 3, 0, start_index);
                    }
                    if (x + 1) >= side_size || !grid[ (z * side_size.pow(2) + y * side_size + x + 1) as usize] {
                        let start_index = (vertices.len() / 6) as u32;
                        add_vertex(&mut vertices, x as f32 + 1.0,   y as f32,       z as f32,       r, 0.0, 0.0, shift);
                        add_vertex(&mut vertices, x as f32 + 1.0,   y as f32 + 1.0, z as f32,       r, 0.0, 0.0, shift);
                        add_vertex(&mut vertices, x as f32 + 1.0,   y as f32 + 1.0, z as f32 + 1.0, r, 0.0, 0.0, shift);
                        add_vertex(&mut vertices, x as f32 + 1.0,   y as f32,       z as f32 + 1.0, r, 0.0, 0.0, shift);
                        add_index(&mut indices, 0, 3, 2, start_index);
                        add_index(&mut indices, 2, 1, 0, start_index);
                    }

                }
            }
        }

        let mut vbo: gl::types::GLuint = 0;
        let mut vao: gl::types::GLuint = 0;
        let mut ebo: gl::types::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER, // target
                (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
                vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );
            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl::VertexAttribPointer(
                0, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null() // offset of the first component
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid // offset of the first component
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER, // target
                (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr, // size of data in bytes
                indices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0); // unbind the buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
        Model { vao, vbo, ebo, indices_count: indices.len() as i32 }
    }
}
impl Drop for Model {
    fn drop(&mut self) {
        unsafe {
            let buffers = [self.vbo, self.ebo];
            gl::DeleteBuffers(2, buffers.as_ptr());
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

fn add_vertex(arr: &mut Vec<f32>, x: f32, y: f32, z: f32, r: f32, g: f32, b: f32, shift: f32) {
    arr.push(x - shift);
    arr.push(y - shift);
    arr.push(z - shift);
    arr.push(r);
    arr.push(g);
    arr.push(b);
}
fn add_index(arr: &mut Vec<u32>, v1: u32, v2: u32, v3: u32, add: u32) {
    arr.push(v1 + add);
    arr.push(v2 + add);
    arr.push(v3 + add);
}

fn build_f_cube(array: &mut Vec<bool>, side_size: i32, x: i32, y: i32, z: i32, size: i32) {
    if size == 0 { return; }
    if size == 1 { array[ (z * side_size.pow(2) + y * side_size + x)  as usize] = true; return; }

    for xc in 0..3i32 {
        for yc in 0..3i32 {
            for zc in 0..3i32 {
                if xc == 1 && yc == 1 || yc == 1 && zc == 1 || zc == 1 && xc == 1
                    { continue };
                build_f_cube(array, side_size, x + xc * 3i32.pow(size as u32 - 2), y + yc * 3i32.pow(size as u32 - 2), z + zc * 3i32.pow(size as u32 - 2), size - 1);
            }
        }
    }
}