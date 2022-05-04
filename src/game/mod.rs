#![allow(dead_code)]

mod traits;
mod utils;
mod atlas;
mod load;

pub use utils::*;
pub use atlas::*;

use std::f64::consts::PI;
use std::path::PathBuf;
use crate::{Input, mat, rgl};
use crate::mat::{Vec3};
use crate::resources::Resources;
use crate::rgl::Model;


const MAX_GRID_SIZE: usize = 128*128*128;
const CHUNK_SIZE: usize     = 32;
const CHUNK_VOLUME: usize   = CHUNK_SIZE.pow(3);


pub struct Player {
    pub x: f64,
    pub y: f64,
    pub z: f64,

    pub ang_vert: f64,
    pub ang_horz: f64,
}
impl Player {
    pub fn new() -> Self {
        Player{
            x: 0.0, y: 0.0, z: 0.0,
            ang_horz: 0.0, ang_vert: 0.0,
        }
    }
    pub fn move_plr(&mut self, forw: f64, right: f64, up: f64) {
        let az = self.ang_horz;        //Угол, указывающий прямо
        let azr = self.ang_horz - PI / 2.0;     //Угол вправо
        let ax = self.ang_vert;
        self.x += az.sin() * ax.cos() * forw - azr.sin() * right;
        self.y += az.cos() * ax.cos() * forw - azr.cos() * right;
        self.z += ax.sin() * forw + up;
    }
    pub fn move_by_input(&mut self, inp: &Input, step: f64) {
        let vel = if inp.key_pressed(sdl2::keyboard::Keycode::LShift) { 80.0 } else { 4.00 };
        let step = step * vel * 1.0;
        let dx = if inp.key_pressed(sdl2::keyboard::Keycode::W) { step } else { 0.0 };
        let dx = if inp.key_pressed(sdl2::keyboard::Keycode::S) { dx - step } else { dx };

        let dy = if inp.key_pressed(sdl2::keyboard::Keycode::D) { step } else { 0.0 };
        let dy = if inp.key_pressed(sdl2::keyboard::Keycode::A) { dy - step } else { dy };

        let dz = if inp.key_pressed(sdl2::keyboard::Keycode::Space) { step } else { 0.0 };
        let dz = if inp.key_pressed(sdl2::keyboard::Keycode::LCtrl) { dz - step } else { dz };

        self.move_plr(dx, dy, dz)
    }

    pub fn set_rotation(&mut self, vert: f64, horz: f64) {
        self.ang_vert = vert;
        self.ang_horz = horz;
    }

    pub fn rotate_by_mouse(&mut self, x: i32, y: i32, sensitivity: f64) {
        self.ang_horz = (self.ang_horz + (x as f64) * sensitivity) % (2.0* std::f64::consts::PI);
        self.ang_vert += (y as f64) * sensitivity;
        self.ang_vert = self.ang_vert.clamp(-std::f64::consts::PI/2.0, std::f64::consts::PI/2.0);
    }
}

/** Шесть сторон куба, аббревиатуры: Positive/Negative X/Y/Z (по нормали к поверхности стороны) */
pub enum BlockFace { PX, NX, PY, NY, PZ, NZ }

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct ShapeVertex {
    pos: Vec3,

    normal: Vec3,       //Нормаль к поверхности к точке.
    tangent_x: Vec3,    //Касательная, указывающая направление +х текстуры
    tangent_y: Vec3,    //Касательная, указывающая направление +y текстуры

    tex_x:  f32,        //Позиция пикселя на текстуре
    tex_y:  f32,
}
impl ShapeVertex {
    fn new(x: f32, y: f32, z: f32, normal_x: f32, normal_y: f32, normal_z: f32, tex_x: f32, tex_y: f32) -> Self {
        ShapeVertex{
            pos: Vec3::new(x, y, z),

            normal: Vec3::new(normal_x, normal_y, normal_z),
            tangent_x: Vec3::new(0.0, 0.0, 0.0),
            tangent_y: Vec3::new(0.0, 0.0, 0.0),

            tex_x, tex_y
        }
    }
    fn to_vertex(&self, offset: Vec3, tangent_x: Vec3, tangent_y: Vec3, tex_id: f32) -> Vertex {
        let mut res = self.clone();
        res.pos += offset;
        res.tangent_x = tangent_x;
        res.tangent_y = tangent_y;
        Vertex::new(res, tex_id)
    }

    fn x(&self) -> f32 { self.pos.x() }
    fn y(&self) -> f32 { self.pos.y() }
    fn z(&self) -> f32 { self.pos.z() }
}

/** Набор полигонов, объединенных одной текстурой */
#[derive(Clone, Debug)]
pub struct BMShape {
    /** Список вершин полигонов (или полигона)*/
    vertices: Vec<ShapeVertex>,
    /** Список индексов */
    indices: Vec<(u32, u32, u32)>,
    /** Настройка того, будет ли полигон отображаться при закрытии различных сторон куба другими блоками.
    Биты обозначают стороны блока. Когда все стороны, обозначенные единицей закрыты - фигура
    перестает отображаться. Стороны идут в порядке: +x, -x, +y, -y, +z, -z, (обозначает нормаль стороны).
     Если все биты (булеаны) равны нулю, то фигура будет отображаться всегда*/
    overlap_state: DenseBools,
}
impl BMShape {
    fn new() -> Self { Self { vertices: vec![], indices: vec![], overlap_state: 0u8.into() } }

    /** Добавить вершину в модель фигуры */
    fn vertex(&mut self, v: ShapeVertex) -> &mut Self {
        self.vertices.push(v);
        self
    }
    /** Добавить вершины в модель фигуры*/
    fn vertices(&mut self, ver: &Vec<ShapeVertex>) -> &mut Self {
        for v in ver { self.vertices.push(*v); };
        self
    }
    /** Добавить треугольник (три индекса) в модель фигуры */
    fn index_u32(&mut self, v1: u32, v2: u32, v3: u32) -> &mut Self {
        self.indices.push((v1, v2, v3));
        self
    }
    /** Добавить треугольник (три индекса) в модель фигуры */
    fn index(&mut self, i: (u32, u32, u32)) -> &mut Self {
        self.indices.push(i);
        self
    }
    /** Добавить треугольники (тройки индексов) в модель фигуры*/
    fn indices(&mut self, ids: &Vec<(u32, u32, u32)>) -> &mut Self {
        for i in ids { self.indices.push(*i); };
        self
    }
    /** Привязать к одной стороне блока*/
    fn set_face(&mut self, face: BlockFace) -> &mut Self {
        self.overlap_state = DenseBools( 1u8 << (face as usize) );
        self
    }
    /** Добавить сторону блока в зависимости */
    fn face(&mut self, face: BlockFace) -> &mut Self {
        self.overlap_state.set(face.into(), true);
        self
    }
    /** Исключить сторону блока из зависимостей*/
    fn not_face(&mut self, face: BlockFace) -> &mut Self {
        self.overlap_state.set(face.into(), false);
        self
    }
    /** Установить overlap значением байта*/
    fn overlap(&mut self, ovl: u8) -> &mut Self {
        self.overlap_state.0 = ovl;
        self
    }
    /** Зависит ли отображение фигуры от закрытости сторон блока*/
    fn is_dependent(&self) -> bool {
        self.overlap_state.0 != 0 //Хотя бы один из битов (булеанов) ненулевой
    }

    /** Просчет геометрии, в частности, кастельных базиса */
    fn calc_geometry(&mut self) {
        //Количество примитивов, связанных с каждой вершиной
        let mut counts: Vec<usize> = [0usize; 1].iter().cycle().take(self.vertices.len()).map(|x| *x).collect();
        for index in self.indices.iter() {
            for k in 0..3 {
                let index_arr = [index.0 as usize, index.1 as usize, index.2 as usize];
                let id0 = index_arr[k];
                let id1 = index_arr[(k + 1) % 3];
                let id2 = index_arr[(k + 2) % 3];
                let (basis_x, basis_y)  = BMShape::get_basis_prim
                    (&self.vertices[id0], &self.vertices[id1], &self.vertices[id2] );

                counts[id0] += 1;
                self.vertices[id0].tangent_x += Vec3::from(basis_x);
                self.vertices[id0].tangent_y += Vec3::from(basis_y);
            }
        }
        for i in 0..self.vertices.len() {
            self.vertices[i].tangent_x /= counts[i] as f32;
            self.vertices[i].tangent_y /= counts[i] as f32;
        }

    }

    fn get_basis_prim(v: &ShapeVertex, sub_v1: &ShapeVertex, sub_v2: &ShapeVertex) -> (mat::Vec4, mat::Vec4) {
        let (ang1, x_1, y_1) = BMShape::get_basis_xy(v, sub_v1);
        let (ang2, x_2, y_2) = BMShape::get_basis_xy(v, sub_v2);

        let tau = std::f32::consts::PI * 2.0;
        let ang1 = ((ang1 % tau) + tau) % tau - std::f32::consts::PI;
        let ang2 = ((ang2 % tau) + tau) % tau - std::f32::consts::PI;

        if ang1.abs() < ang2.abs()  { (x_1, y_2) }
        else                        { (x_2, y_1) }
    }

    /** Возвращает угол между вектором и иксом базиса, также вектор Х базиса в мировом пространстве */
    fn get_basis_xy(vertex1: &ShapeVertex, vertex2: &ShapeVertex) -> (f32, mat::Vec4, mat::Vec4) {
        use mat::{Vec4, Mat4};
        let d_pos = Vec4::from(vertex2.pos - vertex1.pos);
        let edge_normal = Vec4::from((vertex1.normal + vertex2.normal) / 2.0).unit();
        let d_tex_pos = Vec3::new(vertex2.tex_x, vertex2.tex_y, 0.0) - Vec3::new(vertex1.tex_x, vertex1.tex_y, 0.0);

        let ang = d_tex_pos.get_yz_angles().1;
        let tang_x = Mat4::axis_rotation_mat(&edge_normal, -ang) * d_pos;
        let tang_y = Mat4::axis_rotation_mat(&edge_normal, std::f32::consts::PI / 2.0 - ang) * d_pos;
        (ang, tang_x, tang_y)
    }
}

/** Одна вершина */
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    shape_vert: ShapeVertex,
    tex_id: f32,
}
impl Vertex {
    fn new(shape_vert: ShapeVertex, tex_id: f32) -> Self {
        Vertex{ shape_vert, tex_id }
    }
}

/** Набор фигур разных текстур, складывающихся в модельку*/
#[derive(Clone, Debug)]
pub struct BlockModel {
    /** Группы фигур. Все фигуры в пределах группы используют только одну текстуру блока */
    pub shapes: Vec<Vec<BMShape>>,
    /** Обозначение того, закрывают ли разные вершины сторону блока целиком */
    solid_faces: DenseBools,
    /** Имя модели */
    name: String,
}
impl BlockModel {
    pub fn new(name: String) -> Self { BlockModel{ name, shapes: vec![], solid_faces: DenseBools(0) }  }

    /** Все стороны блока покрывают всю площадь сторон*/
    pub fn solid(&mut self) -> &mut Self {
        self.solid_faces = DenseBools(0b00111111);
        self
    }
    /** Установить сторону сплошной */
    pub fn solid_face(&mut self, face: BlockFace) -> &mut Self {
        self.solid_faces.set(face as usize, true);
        self
    }
    /** Установить сторону несплошной */
    pub fn non_solid_face(&mut self, face: BlockFace) -> &mut Self {
        self.solid_faces.set(face as usize, false);
        self
    }
    /** Установить значения всем сторонам */
    pub fn set_faces(&mut self, data: u8) -> &mut Self {
        self.solid_faces = DenseBools(data);
        self
    }

    /** Создать новую группу фигур */
    pub fn new_group(&mut self) -> &mut Self {
        self.shapes.push(vec![]);
        self
    }
    /** Добавить фигуру в текущую группу */
    pub fn add(&mut self, shape: BMShape) -> &mut Self {
        if self.shapes.len() == 0 { self.new_group(); }
        let id = self.shapes.len() - 1;
        let mut shape = shape;
        shape.calc_geometry();
        self.shapes[id].push(shape);
        self
    }
    /** Добавить фигуры в текущую группу */
    pub fn add_shapes(&mut self, shapes: Vec<BMShape>) -> &mut Self {
        if self.shapes.len() == 0 { self.new_group(); }
        for shape in shapes {
            let id = self.shapes.len() - 1;
            let mut shape = shape;
            shape.calc_geometry();
            self.shapes[id].push(shape);
        }
        self
    }

    /** Добавить модель блока в общую (обычно модель чанка) по переданным данным
        vertices        - массив данных вершин модели
        indices         - массив индексов
        textures        - координаты текстур на атласе
        overlap_state   - закрытость блока по сторонам
        pos             - позиция блока
        atlas_size      - размер атласа*/
    pub fn add_to_model(&self, pos: Vec3, overlap_state: u8, vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, textures: &Vec<u32>) {
        if textures.len() < self.shapes.len() { panic!("Not enough textures passed to BlockModel") }

        for (group_id, group) in self.shapes.iter().enumerate() {
            let tex_id = 1.0_f32 / (textures[group_id] as f32 + 1.0);

            for shape in group.iter() {
                //Если фигура перекрыта другими блоками - пропуск
                if shape.is_dependent() && ( overlap_state & shape.overlap_state.0 == shape.overlap_state.0 ) { continue; }

                let start_index = vertices.len() as u32;
                //Добавление вершин
                for v in shape.vertices.iter() {
                    vertices.push(v.to_vertex(pos, Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), tex_id) );
                }
                //Добавление индексов
                for i in shape.indices.iter() {
                    indices.push(i.0 + start_index);
                    indices.push(i.1 + start_index);
                    indices.push(i.2 + start_index);
                }
            }
        }
    }
}

/** Глобальные (в пределах одного Game) данные о блоке */
#[derive(Clone)]
pub struct BlockData {
    pub model_id: usize,     //Номер модели
    pub textures: Vec<u32>,  //Номера текстур, подаваемых в модель

    pub name: String,            //Очевидно, название блока
}

pub enum WaitingAction {
    //id, x, y, z
    SetBlock(u8, i32, i32, i32),
    //dense_faces, x, y, z
    UpdateFaces(u8, i32, i32, i32)
}
/** Структура, хранящая все игровые данные */
pub struct Game {
    atlas: Atlas,   //Атлас текстур и данные о расположении этих текстур

    models:     Vec<BlockModel>,
    model_ids:  Vec<(String, usize)>,
    blocks:     Vec<BlockData>,
    block_ids:  Vec<(String, usize)>,

    pub chunks:       Vec<Chunk>,         //Чанки в неопределенном порядке
    chunk_models: Vec<Option<Model>>, //Модели чанков (чанк может быть пуст)
    chunks_grid:  Vec<Option<usize>>, //Номера чанков по сетке (чанк может быть и не загружен)
    chunks_start: (i32, i32, i32),    //Начало и конец прямоугольного паралеллепипеда чанков
    chunks_end:   (i32, i32, i32),

    //Действие может быть применено к выгруженному чанку, тогда оно попадает в ожидание на выполнение
    waiting_actions: Vec<Option<WaitingAction>>,
}
impl Game {
    pub fn new(res: &Resources) -> Self {
        let mut textures_required: Vec<String> = vec![
            "dirt", "grass_side", "grass_top", "stone",
            "log_side", "log_top", "log_top_cyl", "leaves"
        ].iter().map(|x| x.to_string()).collect();
        let blocks_tmp: Vec<(String, Vec<u32>, String)> = vec![
            ("air".into(),      vec![0u32, 0, 0, 0, 0, 0], "empty".into()),
            ("dirt".into(),     vec![1u32, 1, 1, 1, 1, 1], "cube".into()),
            ("grass".into(),    vec![2u32, 2, 2, 2, 3, 1], "cube".into()),
            ("stone".into(),    vec![4u32, 4, 4, 4, 4, 4], "cube".into()),
            ("log".into(),      vec![5u32, 5, 5, 5, 6, 6], "cube".into()),
            ("log_cyl".into(),  vec![5u32, 7, 7],          "cyl_low".into()),
            ("leaves".into(),   vec![8u32, 8, 8, 8, 8, 8], "cube".into()),
        ];

        let mut atlas = AtlasBuilder::new(15, 15);
        atlas.add_names(&mut textures_required)
            .load_textures(PathBuf::from("textures"), res)
            .load_normals(PathBuf::from("normal_maps"), res)
            .load_lightmaps(PathBuf::from("light_maps"), res);
        let atlas = atlas.build();
        println!("Save: {:?}", atlas.image().save(PathBuf::from("assets/tmp_atlas.png")));
        /*for level in 0..5 {
            let img = generate_mipmap(atlas.image(), (15, 15), level);
            println!("Save {}: {:?}", level, img.save(PathBuf::from(format!("assets/tmp_atlas_{}.png", level))));
        }*/

        let mut result = Self {
            atlas,
            models:         vec![],
            model_ids:      vec![],
            blocks:         vec![],
            block_ids:      vec![],

            chunks:         vec![],
            chunk_models:   vec![],
            chunks_grid:    vec![],
            chunks_start:   (i32::MAX, i32::MAX, i32::MAX),
            chunks_end:     (i32::MIN, i32::MIN, i32::MIN),

            waiting_actions: vec![],
        };

        result.add_model(BlockModel::new("empty".into()))
            .add_model(cube_block_model())
            .add_model(cylinder_block_model());

        for b in blocks_tmp {
            result.add_block(BlockData {
                name: b.0,
                model_id: result.get_model_id(b.2).unwrap(),
                textures: b.1,
            });
        }

        ;result
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        let (cx, cy, cz) = (chunk.x.clone(), chunk.y.clone(), chunk.z.clone());

        let mut changed = false;
        let mut new_start = self.chunks_start.clone();
        if cx < new_start.0 {  new_start.0 = cx; changed = true;  }
        if cy < new_start.1 {  new_start.1 = cy; changed = true;  }
        if cz < new_start.2 {  new_start.2 = cz; changed = true;  }

        let mut new_end = self.chunks_end.clone();
        if cx < new_end.0 {  new_end.0 = cx; changed = true;  }
        if cy < new_end.1 {  new_end.1 = cy; changed = true;  }
        if cz < new_end.2 {  new_end.2 = cz; changed = true;  }

        self.chunks_start = new_start;
        self.chunks_end = new_end;
        self.chunks.push(chunk);

        if changed {
            self.reload_grid();
        } else {
            let chunk_id = self.chunk_id(cx, cy, cz);
            self.chunks_grid[chunk_id] = Some(self.chunks.len() - 1);
        }
    }
    pub fn reload_grid(&mut self) {
        if self.chunks_start.0 > self.chunks_end.0 ||
           self.chunks_start.1 > self.chunks_end.1 ||
           self.chunks_start.2 > self.chunks_end.2  {
            self.chunks_grid = vec![];
            self.chunks_start = (0, 0, 0);
            self.chunks_end = (-1, -1, -1);
        }
        println!("{:?} <> {:?}", self.chunks_start, self.chunks_end);
        let volume = ((self.chunks_end.0 - self.chunks_start.0 + 1) *
                          (self.chunks_end.1 - self.chunks_start.1 + 1) *
                          (self.chunks_end.2 - self.chunks_start.2 + 1)) as usize;

        if volume > MAX_GRID_SIZE {
            panic!("Too large chunks grid");
        }

        self.chunks_grid = [None; 1].iter().cycle().take(volume).map(|x| *x).collect();

        for (i, chunk) in self.chunks.iter().enumerate() {
            if self.is_in_grid(chunk.x, chunk.y, chunk.z) {
                let chunk_id = self.chunk_id(chunk.x, chunk.y, chunk.z);
                self.chunks_grid[chunk_id] = Some(i);
            }
        }
    }

    pub fn get_chunk_id(&mut self, x: i32, y: i32, z: i32) -> Option<usize> {
        if !self.is_in_grid(x, y, z) {
            return None;
        }
        self.chunks_grid[self.chunk_id(x, y, z)]
    }

    pub fn set_block(&mut self, block: u8, x: i32, y: i32, z: i32) {
        //Чанк выгружен, откладываем действие.
        if !self.is_in_grid(x, y, z) {
            self.waiting_actions.push(Some(WaitingAction::SetBlock(block, x, y, z)));
            return;
        }
        let chs = CHUNK_SIZE as i32;
        let chunk_id = self.chunk_id(x / chs, y / chs, z / chs);

        //Чанк выгружен, откладываем действие
        if let None = self.chunks_grid[chunk_id] {
            self.waiting_actions.push(Some(WaitingAction::SetBlock(block, x, y, z)));
        }

        let target_chunk_id = &self.chunks_grid[chunk_id].unwrap();
        let target_chunk = &mut self.chunks[*target_chunk_id];

        let in_chunk_local_pos = (to_chunk_mod(x), to_chunk_mod(y), to_chunk_mod(z));

        //Модель устанавливаемого блока
        let block_model = &self.models[self.blocks[block as usize].model_id];
        let solid_faces = block_model.solid_faces.clone();

        target_chunk.set_block(block, in_chunk_local_pos.0 as i32, in_chunk_local_pos.1 as i32, in_chunk_local_pos.2 as i32, solid_faces);
        for dx in -1..2 {
            for dy in -1..2 {
                for dz in -1..2 {
                    if let Some(id) = self.get_chunk_id(dx + x / chs, dy + y / chs, dz + z / chs) {
                        self.chunks[id].update_faces(in_chunk_local_pos.0 as i32 - dx * chs,
                                                     in_chunk_local_pos.1 as i32 - dy * chs,
                                                     in_chunk_local_pos.2 as i32 - dz * chs,
                                                     solid_faces);
                    }
                }
            }
        }
    }

    pub fn update_faces(&mut self, solid_faces: DenseBools, x: i32, y: i32, z: i32) {
        let chs = CHUNK_SIZE as i32;
        let chunk_id = self.chunk_id(x / chs, y / chs, z / chs);

        let target_chunk_id = &self.chunks_grid[chunk_id].unwrap();
        let in_chunk_local_pos = (to_chunk_mod(x), to_chunk_mod(y), to_chunk_mod(z));

        let gxs = self.grid_x_size();
        let gys = self.grid_y_size();
        let _gzs = self.grid_z_size();
        //Позиция чанка в сетке
        let chunk_x =  target_chunk_id % gxs;
        let _chunk_y = (target_chunk_id / gxs) % gys;
        let _chunk_z =  target_chunk_id / gxs  / gys;

        if chunk_x > 0 {
            //Чанк со стороны -х от данного
            if let Some(id) = self.chunks_grid[target_chunk_id - 1] {
                self.chunks[id].update_faces((in_chunk_local_pos.0 + CHUNK_SIZE) as i32,
                                             in_chunk_local_pos.1 as i32, in_chunk_local_pos.2 as i32, solid_faces);
            } else {
                self.waiting_actions.push(Some(
                    WaitingAction::UpdateFaces(solid_faces.0, x, y, z)
                ));
            }
        }

        if chunk_x < (gxs - 1) {
            //Чанк со стороны +х от данного
            if let Some(id) = self.chunks_grid[target_chunk_id + 1] {
                self.chunks[id].update_faces((in_chunk_local_pos.0 - CHUNK_SIZE) as i32,
                                             in_chunk_local_pos.1 as i32, in_chunk_local_pos.2 as i32, solid_faces);
            } else {
                self.waiting_actions.push(Some(
                    WaitingAction::UpdateFaces(solid_faces.0, x, y, z)
                ));
            }
        }
        todo!();
    }

    pub fn is_in_grid(&self, x: i32, y: i32, z: i32) -> bool {
        if  self.chunks_start.0 <= x &&
            self.chunks_start.1 <= y &&
            self.chunks_start.2 <= z &&
            self.chunks_end.0 >= z &&
            self.chunks_end.1 >= y &&
            self.chunks_end.2 >= z
        { true }
        else
        { false }
    }

    pub fn add_model(&mut self, model: BlockModel) -> &mut Self {
        if let Ok(_) = self.get_model_id(model.name.clone()) {
            println!("Model {} already exists", model.name);
            return self;
        }
        self.model_ids.push((model.name.clone(), self.models.len()));
        self.models.push(model);
        self
    }
    pub fn add_block(&mut self, block: BlockData) -> &mut Self {
        if let Ok(_) = self.get_block_id(block.name.clone()) {
            println!("Block {} already exists", block.name);
            return self;
        }
        self.block_ids.push((block.name.clone(), self.blocks.len()));
        self.blocks.push(block);
        self
    }

    pub fn get_model_id(&self, name: String) -> Result<usize, ()> {
        for (n, i) in self.model_ids.iter() {
            if n == &name { return Ok(*i); }
        }
        Err(())
    }
    pub fn get_block_id(&self, name: String) -> Result<usize, ()> {
        for (n, i) in self.block_ids.iter() {
            if n == &name { return Ok(*i); }
        }
        Err(())
    }

    fn chunk_id(&self, x: i32, y: i32, z: i32) -> usize {
        (x - self.chunks_start.0) as usize +
        (y - self.chunks_start.1) as usize * self.grid_x_size() +
        (z - self.chunks_start.2) as usize * self.grid_x_size() * self.grid_y_size()
    }

    fn grid_x_size(&self) -> usize { (self.chunks_end.0 - self.chunks_start.0) as usize }
    fn grid_y_size(&self) -> usize { (self.chunks_end.1 - self.chunks_start.1) as usize }
    fn grid_z_size(&self) -> usize { (self.chunks_end.2 - self.chunks_start.2) as usize }

    pub fn models(&self) -> &Vec<BlockModel> { &self.models }
    pub fn blocks(&self) -> &Vec<BlockData> { &self.blocks }
    pub fn atlas(&self) -> &Atlas { &self.atlas }
}

pub struct Chunk {
    //ID, закрытость другими блоками
    pub x: i32,
    pub y: i32,
    pub z: i32,

    changed: bool,
    blocks_count: usize,

    data: Vec<(u8, DenseBools)>,
}
impl Chunk {
    pub fn empty(x: i32, y: i32, z: i32) -> Self {
        Self {
            x, y, z,
            changed: false,
            blocks_count: 0,
            data: [(0u8,DenseBools(0u8)); 1].iter().cycle().take(CHUNK_VOLUME).map(|x| *x).collect()
        }
    }

    pub fn set_block(&mut self, block: u8, x: i32, y: i32, z: i32, block_solidness: DenseBools) {
        let chs = CHUNK_SIZE as i32;
        if  x >= 0 && x < chs &&
            y >= 0 && y < chs &&
            z >= 0 && z < chs {
            self.data[Chunk::pos_id(x as usize, y as usize, z as usize)].0 = block;
        }
        self.update_faces(x, y, z, block_solidness);
    }

    pub fn update_faces(&mut self, x: i32, y: i32, z: i32, block_solidness: DenseBools) {
        self.changed = true;
        let chs = CHUNK_SIZE as i32;
        if x < chs - 1 && x >= -1 {     //Блоку на x + 1 нужно отметить закрытость стороны
            self.data[Chunk::pos_id(x as usize + 1, y as usize, z as usize)].1
                .set(BlockFace::NX.into(), block_solidness.get(BlockFace::PX.into()) );
        }
        if x > 0 && x <= chs {     //Блоку на x - 1 нужно отметить закрытость стороны
            self.data[Chunk::pos_id(x as usize - 1, y as usize, z as usize)].1
                .set(BlockFace::PX.into(), block_solidness.get(BlockFace::NX.into()));
        }
        if y < chs - 1 && y >= -1 {     //Блоку на y + 1 нужно отметить закрытость стороны
            self.data[Chunk::pos_id(x as usize, y as usize + 1, z as usize)].1
                .set(BlockFace::NY.into(), block_solidness.get(BlockFace::PY.into()));
        }
        if y > 0 && y <= chs {     //Блоку на y - 1 нужно отметить закрытость стороны
            self.data[Chunk::pos_id(x as usize, y as usize - 1, z as usize)].1
                .set(BlockFace::PY.into(), block_solidness.get(BlockFace::NY.into()));
        }
        if z < chs - 1 && z >= -1 {     //Блоку на z + 1 нужно отметить закрытость стороны
            self.data[Chunk::pos_id(x as usize, y as usize, z as usize + 1)].1
                .set(BlockFace::NZ.into(), block_solidness.get(BlockFace::PZ.into()));
        }
        if z > 0 && z <= chs {     //Блоку на z - 1 нужно отметить закрытость стороны
            self.data[Chunk::pos_id(x as usize, y as usize, z as usize - 1)].1
                .set(BlockFace::PZ.into(), block_solidness.get(BlockFace::NZ.into()));
        }
    }

    pub fn build_model(&self, blocks_data: &Vec<BlockData>, models_data: &Vec<BlockModel>) -> rgl::Model {
        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u32> = vec![];

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let id = Chunk::pos_id(x, y, z);
                    if self.data[id].0 == 0 { continue; }
                    models_data[blocks_data[self.data[id].0 as usize].model_id].add_to_model(
                        mat::Vec3::new(x as f32, y as f32, z as f32),
                        self.data[id].1.0,
                        &mut vertices, &mut indices,
                        &blocks_data[self.data[id].0 as usize].textures,
                    );
                }
            }
        }

        use AttribType::*;
        //Position, Normal, Tangent X, Tangent Y, Tex Coordinates, Texture_ID
        let attributes: Vec<AttribType> = vec![Vec3, Vec3, Vec3, Vec3, Vec2, Float];
        texture_model(&vertices, &indices, &attributes)
    }

    pub fn pos_id(x: usize, y: usize, z: usize) -> usize {
        z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x
    }

}

fn to_chunk_mod(i: i32) -> usize {
    let chs = CHUNK_SIZE as i32;
    (((i % chs) + chs) % chs) as usize
}

