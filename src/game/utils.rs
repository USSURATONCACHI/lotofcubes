use std::ops::Add;
use std::path::PathBuf;
use crate::game::{BlockFace, BlockModel, BMShape, ShapeVertex, Vertex};
use crate::{Model, rgl};

/** Восемь bool значений, скомпресованные в байт */
#[derive(Clone, Copy, Debug)]
pub struct DenseBools(pub u8);
impl DenseBools {
    fn new() -> Self { Self(0) }

    pub fn from8(vals: [bool; 8]) -> Self {
        let mut res: u8 = 0;
        for (i, b) in vals.iter().enumerate() { res += if *b { 1 << i } else { 0 } }
        Self( res )
    }

    pub fn get(&self,     i: usize) -> bool { self.0 & (1u8 << i) > 0u8 }
    pub fn set(&mut self, i: usize, state: bool) {
        let s: u8 = if state { 1u8 } else { 0u8 } << i;
        self.0 = self.0 ^ ( (self.0 & (1u8 << i)) ^ s );
    }
}

pub enum AttribType {
    Vec3, Vec2, Float, Basis, Int
}
impl AttribType {
    pub fn size(&self) -> usize {
        match self {
            Self::Vec3 =>   { std::mem::size_of::<f32>() * 3 }
            Self::Vec2 =>   { std::mem::size_of::<f32>() * 2 }
            Self::Float =>  { std::mem::size_of::<f32>() * 1 }
            Self::Basis =>  { std::mem::size_of::<f32>() * 9 }
            Self::Int =>    { std::mem::size_of::<i32>() * 1 }
        }
    }
    pub fn data_type(&self) -> (i32, gl::types::GLuint) {
        match self {
            Self::Vec3 =>   { (3, gl::FLOAT) }
            Self::Vec2 =>   { (2, gl::FLOAT) }
            Self::Float =>  { (1, gl::FLOAT) }
            Self::Basis =>  { (9, gl::FLOAT) }
            Self::Int =>    { (1, gl::INT) }
        }
    }
    pub fn is_int(&self) -> bool {
        match self {
            Self::Int => true,
            _ => false,
        }
    }
}

pub fn texture_model(vertices: &Vec<Vertex>, indices: &Vec<u32>, attribs: &Vec<AttribType>) -> Model {
    let mut vbo: gl::types::GLuint = 0;
    let mut vao: gl::types::GLuint = 0;
    let mut ebo: gl::types::GLuint = 0;
    let stride = {
        let mut res = 0;
        for a in attribs.iter() { res += a.size(); }
        res
    } as gl::types::GLint;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::BufferData(gl::ARRAY_BUFFER,
                       (vertices.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr,
                       vertices.as_ptr() as *const gl::types::GLvoid,
                       gl::STATIC_DRAW,
        );

        let mut offset = 0;
        for (i, attr) in attribs.iter().enumerate() {
            let (numbers_count, data_type) : (i32, gl::types::GLuint) = attr.data_type();
            gl::EnableVertexAttribArray(i as u32);
            if attr.is_int() {
                gl::VertexAttribIPointer( i as u32, numbers_count, data_type,
                                         stride, offset as *const gl::types::GLvoid);
            } else {
                gl::VertexAttribPointer( i as u32, numbers_count, data_type,
                                         gl::FALSE, stride,
                                         offset as *const gl::types::GLvoid);
            }
            offset += attr.size();
        }

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                       (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr, // size of data in bytes
                       indices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                       gl::STATIC_DRAW, // usage
        );

        gl::BindVertexArray(0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0); // unbind the buffer
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }
    rgl::Model::create(vao, vbo, ebo, indices.len() as i32)
}

pub fn cube_block_model() -> BlockModel {
    let cube_vertices: [ShapeVertex; 4*6] = [
        ShapeVertex::new(0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 1.0, 1.0),
        ShapeVertex::new(0.5,  0.5, -0.5, 1.0, 0.0, 0.0, 0.0, 1.0),
        ShapeVertex::new(0.5,  0.5,  0.5, 1.0, 0.0, 0.0, 0.0, 0.0),
        ShapeVertex::new(0.5, -0.5,  0.5, 1.0, 0.0, 0.0, 1.0, 0.0), //+X

        ShapeVertex::new(-0.5, -0.5, -0.5, -1.0, 0.0, 0.0, 0.0, 1.0),
        ShapeVertex::new(-0.5,  0.5, -0.5, -1.0, 0.0, 0.0, 1.0, 1.0),
        ShapeVertex::new(-0.5,  0.5,  0.5, -1.0, 0.0, 0.0, 1.0, 0.0),
        ShapeVertex::new(-0.5, -0.5,  0.5, -1.0, 0.0, 0.0, 0.0, 0.0), //-X

        ShapeVertex::new(-0.5,  0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 1.0),
        ShapeVertex::new( 0.5,  0.5, -0.5, 0.0, 1.0, 0.0, 1.0, 1.0),
        ShapeVertex::new( 0.5,  0.5,  0.5, 0.0, 1.0, 0.0, 1.0, 0.0),
        ShapeVertex::new(-0.5,  0.5,  0.5, 0.0, 1.0, 0.0, 0.0, 0.0), //+Y

        ShapeVertex::new(-0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.0, 1.0),
        ShapeVertex::new( 0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 1.0, 1.0),
        ShapeVertex::new( 0.5, -0.5,  0.5, 0.0, -1.0, 0.0, 1.0, 0.0),
        ShapeVertex::new(-0.5, -0.5,  0.5, 0.0, -1.0, 0.0, 0.0, 0.0), //-Y

        ShapeVertex::new(-0.5, -0.5,  0.5, 0.0, 0.0, 1.0, 0.0, 1.0),
        ShapeVertex::new( 0.5, -0.5,  0.5, 0.0, 0.0, 1.0, 1.0, 1.0),
        ShapeVertex::new( 0.5,  0.5,  0.5, 0.0, 0.0, 1.0, 1.0, 0.0),
        ShapeVertex::new(-0.5,  0.5,  0.5, 0.0, 0.0, 1.0, 0.0, 0.0), //+Z

        ShapeVertex::new(-0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 1.0),
        ShapeVertex::new( 0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 1.0),
        ShapeVertex::new( 0.5,  0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 0.0),
        ShapeVertex::new(-0.5,  0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 0.0), //Квадрат -Z
    ];
    let cube_indices: [(u32, u32, u32); 2 * 6] = [
        (2,  3,  0),  (0,  1,  2),
        (2,  1,  0),  (0,  3,  2),
        (2,  1,  0),  (0,  3,  2),
        (2,  3,  0),  (0,  1,  2),
        (2,  3,  0),  (0,  1,  2),
        (2,  1,  0),  (0,  3,  2),
    ];

    let mut cube = BlockModel::new("cube".into());
    cube.solid();
    for face in 0..6 {
        let mut shape = BMShape::new();
        for i in 0..4 {
            let v = cube_vertices[face * 4 + i];
            shape.vertex(v);
        }
        shape.index(cube_indices[face * 2 + 0])
            .index(cube_indices[face * 2 + 1])
            .set_face(face.into());
        cube.add(shape);
        if face < 5 { cube.new_group(); }
    }
    cube
}
pub fn cylinder_block_model() -> BlockModel {
    let s2 = 2.0_f32.sqrt() / 4.0;
    let on = 0.5_f32;
    let zr = 0.0_f32;
    let positions: Vec<(f32, f32, f32)> = vec![
        //Нижняя плоскость
        (zr, -on, -on), (s2, -s2, -on), (on,  zr, -on), (s2,  s2, -on),
        (zr,  on, -on), (-s2, s2, -on), (-on, zr, -on), (-s2,-s2, -on),
        //Верхняя плоскость
        (zr, -on,  on), (s2, -s2,  on), (on,  zr,  on), (s2,  s2,  on),
        (zr,  on,  on), (-s2, s2,  on), (-on, zr,  on), (-s2,-s2,  on),
    ];

    let mut model = BlockModel::new("cyl_low".into());

    //Для боковых граней нормаль на вершине совпадает с позицией вершины по xy
    let indices: Vec<(u32, u32, u32)> = vec![(0, 1, 3), (1, 4, 3), (1, 2, 4), (2, 5, 4)];
    let overlaps: Vec<u8> = vec![0b00110110, 0b00110101, 0b00111001, 0b00111010];
    for side in 0..4 {
        let mut side_shape: BMShape = BMShape::new();
        for z_side in 0..2 {
            for vert_id in 0..3 {
                let pos_id = z_side * 8 + ((side * 2 + vert_id) % 8);
                let pos = positions[pos_id];
                let vertex: ShapeVertex = ShapeVertex::new(pos.0, pos.1, pos.2,
                                                           pos.0, pos.1, 0.0,
                                                           (vert_id as f32) / 2.0, z_side as f32);
                side_shape.vertex(vertex);
            }
        }
        side_shape.indices(&indices);
        side_shape.overlap_state = DenseBools(overlaps[side]);
        model.add(side_shape);
    }

    model.new_group();
    let mut shape_pz = BMShape::new();
    shape_pz.set_face(BlockFace::PZ);
    for i in 8..16 {
        let pos = positions[i];
        shape_pz.vertex(ShapeVertex::new(
            pos.0, pos.1, pos.2,
            0.0, 0.0, 1.0,
            pos.0 + 0.5, pos.1 + 0.5));
    }
    for i in 2..8 {
        shape_pz.index_u32(0, i - 1, i % 8);
    }
    model.add(shape_pz);

    model.new_group();
    let mut shape_nz = BMShape::new();
    shape_nz.set_face(BlockFace::NZ);
    for i in 0..8 {
        let pos = positions[i];
        shape_nz.vertex(ShapeVertex::new(
            pos.0, pos.1, pos.2,
            0.0, 0.0, -1.0,
            pos.0 + 0.5, pos.1 + 0.5));
    }
    for i in 2..8 {
        shape_nz.index_u32(i % 8, i - 1, 0);
    }
    model.add(shape_nz);

    model
}

pub fn get_first_word(path: PathBuf) -> String {
    let words: Vec<&str> = path.to_str().unwrap().split("\\").collect();
    let parts: Vec<&str> = words[0].split(".").collect();

    if parts.len() == 0 { return String::new(); }
    if parts.len() == 1 { return parts[0].into(); }

    let mut res = String::new();
    for i in 0..(parts.len() - 1) {
        if i > 0 { res = res.add("."); }
        res = res.add(parts[i]);
    }
    res
}