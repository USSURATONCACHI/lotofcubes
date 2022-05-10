#![windows_subsystem = "windows"]

extern crate sdl2;
extern crate gl;
extern crate core;

use std::f32::consts::PI;
use std::path::Path;
use std::time::Duration;
use image::DynamicImage;
use sdl2::event::{Event, WindowEvent};
use sdl2::video::{GLContext, SwapInterval, Window};
use sdl2::VideoSubsystem;
use crate::game::DenseBools;
use crate::input::Input;
use crate::mat::Mat4;
use crate::rgl::{Model, Program};

pub mod rgl;
pub mod resources;
pub mod mat;
pub mod input;
pub mod util;
pub mod game;
pub mod glsl_expand;
/*
TODO:
Предупреждение о повторяющихся юнифомах в Program
Нормальную иерархию рендера в мейне
Нормальный лог ошибок и варнингов в glsl_extend
В нем же правку директивы #version
*/
fn main() {
    let mut input = input::Input::new();
    let mut res = resources::Resources::from_relative(Path::new("assets")).unwrap();

    let mut window_data = WindowData::create_window("A lot of cubes", 800, 600);

    let geometry_pass = Program::from_res(&mut res, "shaders/deferred_rendering/geometry_pass",
      vec![
          "u_projview", "u_model", "u_light_projview",
          "u_materials", "u_texture_atlas",
          "u_atlas_size", "u_texture_size",
          "u_light_direction", "u_camera_pos",
      ]).unwrap();

    let lighting_pass = Program::from_res(&mut res, "shaders/deferred_rendering/lighting_pass",
      vec![
          "g_position", "g_normal", "g_color", "g_light",
          "u_light_direction", "u_camera_pos",
      ]).unwrap();

    let mut plr: game::Player = game::Player::new();
    let game = game::Game::new(&res);

    geometry_pass.set_used();
    game.atlas().load_materials_to_shader(&geometry_pass, "u_materials");

    //atlas_size
    geometry_pass.uniform2f(5, game.atlas().width() as f32, game.atlas().height() as f32);
    //texture_size
    geometry_pass.uniform2f(6, game.atlas().tex_width() as f32, game.atlas().tex_height() as f32);

    let _ = load_to_gpu_with_mipmaps(0, game.atlas().image(), 4, (15, 15));
    geometry_pass.uniform1i(4, 0);

    let max_dist = 12;
    let (chunks, blocks) = tmp_create_models(max_dist, &game);
    let mut chunk_model_ids: Vec<usize> = Vec::with_capacity(chunks.len());
    let mut block_model_ids: Vec<usize> = Vec::with_capacity(blocks.len());

    let mut render_dist = 6usize;


    ////////
    let depth_map_width = 2048_i32;
    let depth_map_height = 2048_i32;
    //let (depth_map_fbo, depth_map_texture) = generate_depth_map(depth_map_width, depth_map_height);
    let light_proj_mat = {
        let near: f32 = -500.0;
        let far: f32 = 500.0;
        let ortho_width = 40.0_f32;
        mat::Mat4::orthographic_mat(-ortho_width, ortho_width, -ortho_width, ortho_width, far, near)
    };/*
    shadow_program.set_used();
    shadow_program.uniform_mat4(0, &light_proj_mat);*/


    let mut models_list = ModelList::new();
    for (_, model) in chunks.iter() { chunk_model_ids.push(models_list.add_model(model)); }
    for model in blocks.iter() { block_model_ids.push(models_list.add_model(model)); }

    {
        for cx in 0..max_dist {
            for cy in 0..max_dist {
                let chunk = &chunks[(cy * max_dist + cx) as usize].0;
                let id = chunk_model_ids[(cy * max_dist + cx) as usize];
                let object = mat::Mat4::object_mat(32.0 * (chunk.x as f32), 3.0 + 32.0 * (chunk.y as f32), 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
                models_list.place_object(id, object);
            }
        }
        let step = 3.0_f32.sqrt();
        for (i, _) in blocks.iter().enumerate() {
            let id = block_model_ids[i];
            let object = mat::Mat4::object_mat(i as f32 * step, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
            models_list.place_object(id, object);
        }
    }

    let mut prev_frame: f64 = current_time();

    ////////
    let (g_framebuffer, g_position, g_normal, g_color, g_light) = gen_framebuffer(1366, 768);

    let fullscreen_square = tmp_display_model();

    let mut fps_counter = util::TickCounter::new(30);
    let mut event_pump = window_data.sdl.event_pump().unwrap();
    'main: loop {
        //Как только кадр начался - выводим уже готовый
        window_data.window.gl_swap_window();
        //После - рисуем следующий и ждем
        let frame_start = current_time();

        // Обработка ввода
        for event in event_pump.poll_iter() {
            input.event(event.clone());
            window_data.handle_event(event.clone());
            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                sdl2::event::Event::MouseMotion {xrel, yrel, ..} => {
                    if window_data.is_cursor_captured() { plr.rotate_by_mouse(xrel, -yrel, 0.004); }
                }
                _ => {}
            }
        }
        window_data.handle_input(&mut input);
        plr.move_by_input(&input, frame_start - prev_frame);
        if input.on_pressed(sdl2::keyboard::Keycode::Up, 1) && render_dist < max_dist as usize && window_data.is_cursor_captured() { render_dist += 1; }
        if input.on_pressed(sdl2::keyboard::Keycode::Down, 2) && window_data.is_cursor_captured() && render_dist > 0 { render_dist -= 1; }


        //Включение моделей в рендер
        for cx in 0..render_dist {
            for cy in 0..render_dist {
                models_list.set_renderable(chunk_model_ids[cy * (max_dist as usize) + cx]);
            }
        }
        for (i, _) in blocks.iter().enumerate() { models_list.set_renderable(block_model_ids[i]); }

        //Немного освещения
        let light_direction = (-PI / 4.0, timed_ang(0.1));
        let light_vec = mat::Mat4::rotation_mat(light_direction.0, 0.0, light_direction.1) * mat::Vec4::new(0.0, 1.0, 0.0, 0.0);
        let light_view_mat = mat::Mat4::cam_mat(light_direction.0, -light_direction.1, plr.x as f32, plr.y as f32, plr.z as f32);
        /*shadow_program.set_used();
        shadow_program.uniform_mat4(1, &light_view_mat);*/
        let light_projview = light_proj_mat * light_view_mat;

        unsafe {
            /*shadow_program.set_used();
            gl::Viewport(0, 0, depth_map_width, depth_map_height);
            gl::BindFramebuffer(gl::FRAMEBUFFER, depth_map_fbo);
            gl::Clear(gl::DEPTH_BUFFER_BIT);
            models_list.render_all(&shadow_program, 2);*/

            let view_mat = mat::Mat4::cam_mat(plr.ang_vert as f32, plr.ang_horz as f32, plr.x as f32, plr.y as f32, plr.z as f32);
            let proj_mat = mat::Mat4::perspective_mat(std::f32::consts::PI / 2.0, (window_data.width() as f32) / (window_data.height() as f32), 0.05, 1024.0);

            //Geometry pass
            gl::BindFramebuffer(gl::FRAMEBUFFER, g_framebuffer);
            gl::Viewport(0, 0, 1366, 768);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            geometry_pass.set_used();
            let u_projview = proj_mat * view_mat;
            geometry_pass.uniform_mat4(0, &u_projview);
            geometry_pass.uniform_mat4(2, &light_projview);

            geometry_pass.uniform3f(7, light_vec.x, light_vec.y, light_vec.z);
            geometry_pass.uniform3f(8, plr.x as f32, plr.y as f32, plr.z as f32);
            models_list.render_all(&geometry_pass, 1);
            models_list.finish_render();

            //Lighting pass
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(0, 0, window_data.width as i32, window_data.height as i32);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, g_position);

            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, g_normal);

            gl::ActiveTexture(gl::TEXTURE3);
            gl::BindTexture(gl::TEXTURE_2D, g_color);

            gl::ActiveTexture(gl::TEXTURE4);
            gl::BindTexture(gl::TEXTURE_2D, g_light);

            lighting_pass.set_used();
            lighting_pass.uniform1i(0, 1);
            lighting_pass.uniform1i(1, 2);
            lighting_pass.uniform1i(2, 3);
            lighting_pass.uniform1i(3, 4);

            lighting_pass.uniform3f(4, light_vec.x, light_vec.y, light_vec.z);
            lighting_pass.uniform3f(5, plr.x as f32, plr.y as f32, plr.z as f32);

            fullscreen_square.render();
        }

        prev_frame = frame_start;
        fps_counter.tick();
        //_ - результат этого действия не важен здесь
        let _ = window_data.window.set_title(&format!("A lot of cubes | FPS: {:.2} | XYZ: {:.2}, {:.2}, {:.2} | VH: {:.2}, {:.2}", fps_counter.tps_corrected(), plr.x, plr.y, plr.z, plr.ang_vert, plr.ang_horz)[..]);

        /* Спим до начала следующего кадра.
        Это нужно, поскольку VSync от SDL2 дико грузит процессор вхолостую,
        так что нужно также и вручную следить за ФПС*/
        let fps = window_data.get_monitor_refresh_rate();
        //0.95 - коэффициент. Сон проиходит немного не до конца, чтобы был запас времени на вывод кадра
        let sleep_time = ((1.0 / (fps as f64)) - current_time() + frame_start) * 0.95;
        if sleep_time > 0.0 {
            std::thread::sleep(Duration::from_nanos( (sleep_time * 1_000_000_000.0) as u64 ));
        }
    }
}

struct ModelList<'a> {
    models: Vec<(bool, &'a Model, Mat4)>, //Model and object matrix
}
impl<'a> ModelList<'a> {
    pub fn new() -> Self {  Self{ models: vec![] }  }
    pub fn add_model(&mut self, m: &'a Model) -> usize {
        self.models.push((true, m, Mat4::new(1.0)));
        self.models.len() - 1
    }
    pub fn place_object(&mut self, id: usize, matrix: Mat4) {
        self.models[id].0 = true;
        self.models[id].2 = matrix;
    }
    pub fn set_renderable(&mut self, id: usize) {
        self.models[id].0 = true;
    }

    pub fn render_all(&mut self, program: &Program, object_uniform_id: usize) {
        for (is_renderable, model, matrix) in &mut self.models {
            if *is_renderable {
                program.uniform_mat4(object_uniform_id, matrix);
                model.render();
            }
        }
    }

    pub fn finish_render(&mut self) {
        for (is_renderable, _, _) in &mut self.models { *is_renderable = false; }
    }
}

/** Небольшая обертка, хранящая все нужные данные для работы с выводом в окно */
struct WindowData {
    sdl: sdl2::Sdl,
    video_subsystem: VideoSubsystem,
    window: Window,
    #[allow(dead_code)]
    gl_context: GLContext,

    width: u32,
    height: u32,
    cursor_capture: bool,
}
impl WindowData {
    fn create_window(title: &str, w: u32, h: u32) -> Self {
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();

        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 5);
        //gl_attr.set_multisample_samples(4);
        gl_attr.set_accelerated_visual(true);

        let window = video_subsystem
            .window(title, w, h)
            .opengl()
            .resizable()
            .build()
            .unwrap();

        let gl_context = window.gl_create_context().unwrap();
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

        if let Err(e) = video_subsystem.gl_set_swap_interval(SwapInterval::VSync) {
            panic!("Video subsystem error (VSync): {}", e);
        }

        unsafe {
            gl::Viewport(0, 0, w as i32, h as i32);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::DEPTH_TEST);
            //gl::Enable(gl::MULTISAMPLE);
        }
        Self { sdl, video_subsystem, window, gl_context, width: w, height: h, cursor_capture: false }
    }

    /** Захват курсора в окне */
    fn capture_cursor(&mut self) {
        self.cursor_capture = true;
        self.sdl.mouse().show_cursor(false);
        self.sdl.mouse().capture(true);
    }
    /** Высвобождение курсора */
    fn free_cursor(&mut self) {
        self.cursor_capture = false;
        self.sdl.mouse().show_cursor(true);
        self.sdl.mouse().capture(false);
    }
    /** Захвачен ли курсор прямо сейчас*/
    fn is_cursor_captured(&self) -> bool {
        self.cursor_capture
    }
    /** Частота обновления монитора, на котором расположено окно */
    fn get_monitor_refresh_rate(&self) -> i32 {
        let di = self.window.display_index().unwrap();
        self.video_subsystem.current_display_mode(di).unwrap().refresh_rate
    }

    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }

    fn handle_event(&mut self, e: Event) {
        match e {
            sdl2::event::Event::MouseButtonDown { .. } => {
                self.capture_cursor()
            }
            sdl2::event::Event::MouseMotion {x, y, ..} => {
                if self.is_cursor_captured() {
                    let w = self.width() as i32;
                    let h = self.height() as i32;
                    if x < (w / 4) || x >= (w * 3 / 4) || y < (h / 4) || y >= (h * 3 / 4) {
                        self.sdl.mouse().warp_mouse_in_window(&self.window, w / 2, h / 2);
                    }
                }
            }
            sdl2::event::Event::Window {win_event, ..} => {
                if let WindowEvent::Resized(w, h) = win_event {
                    self.width = w as u32;
                    self.height = h as u32;
                    unsafe { gl::Viewport(0, 0, w, h); }
                }
            }
            _ => {}
        }
    }

    fn handle_input(&mut self, input: &mut Input) {
        if (
            input.on_pressed(sdl2::keyboard::Keycode::Escape, 0) ||
                (self.window.window_flags() & (1 << 9)) == 0  //Окно свернуто, переключено и т.д.
        ) && self.is_cursor_captured() {
            self.free_cursor();
        }
    }
}

fn timed_ang(multiplier: f64) -> f32 {
    (current_time() * multiplier % (2.0 * std::f64::consts::PI)) as f32
}

fn current_time() -> f64 {
    (
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as f64
    ) / 1000000.0
}

/** Возвращает FBO и текстуру */
fn generate_depth_map(w: i32, h: i32) -> (u32, u32) {
    let mut depth_map_fbo: u32 = 0;
    let mut depth_map: u32 = 0;
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0 + 1);
        gl::GenFramebuffers(1, &mut depth_map_fbo);
        gl::GenTextures(1, &mut depth_map);
        gl::BindTexture (gl::TEXTURE_2D, depth_map);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as gl::types::GLint, w, h, 0, gl::DEPTH_COMPONENT, gl::FLOAT, std::ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as gl::types::GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as gl::types::GLint);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as gl::types::GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as gl::types::GLint);
        let border_depth: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, border_depth.as_ptr());
        /*
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_BORDER);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_BORDER);
        float borderColor[] = { 1.0f, 1.0f, 1.0f, 1.0f };
        glTexParameterfv(GL_TEXTURE_2D, GL_TEXTURE_BORDER_COLOR, borderColor); */

        gl::BindTexture (gl::TEXTURE_2D, 0);

        gl::BindFramebuffer     (gl::FRAMEBUFFER, depth_map_fbo);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_map, 0);
        gl::DrawBuffer          (gl::NONE);
        gl::ReadBuffer          (gl::NONE);
        gl::BindFramebuffer     (gl::FRAMEBUFFER, 0);
    }

    (depth_map_fbo, depth_map)
}

fn gen_framebuffer(width: u32, height: u32) -> (u32, u32, u32, u32, u32) {
    let width = width as i32;
    let height = height as i32;
    unsafe {
        unsafe fn gen_buffer(width: i32, height: i32, internal_format: i32, format: u32, data_type: u32, attachment: u32) -> u32 {
            let mut buffer = 0;
            gl::GenTextures(1, &mut buffer);
            gl::ActiveTexture(gl::TEXTURE15);
            gl::BindTexture(gl::TEXTURE_2D, buffer);
            gl::TexImage2D(gl::TEXTURE_2D, 0, internal_format, width, height, 0, format, data_type, std::ptr::null());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachment, gl::TEXTURE_2D, buffer, 0);
            buffer
        }
        let mut g_buffer: u32 = 0;
        gl::GenFramebuffers(1, &mut g_buffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, g_buffer);
        // Буффер позиций фрагментов
        let g_position: u32  = gen_buffer(width, height, gl::RGB32F as i32, gl::RGB, gl::FLOAT, gl::COLOR_ATTACHMENT0);
        // Буффер нормалей фрагментов
        let g_normal: u32    = gen_buffer(width, height, gl::RGB32F as i32, gl::RGB, gl::FLOAT, gl::COLOR_ATTACHMENT1);
        // Буффер цветов фрагментов
        let g_color: u32     = gen_buffer(width, height, gl::RGB as i32, gl::RGB, gl::UNSIGNED_BYTE, gl::COLOR_ATTACHMENT2);
        // Буффер взаимодействия фрагментов со светом
        let g_light: u32     = gen_buffer(width, height, gl::RGB as i32, gl::RGB, gl::FLOAT, gl::COLOR_ATTACHMENT3);

        // буффер глубины
        let _g_depth: u32     = gen_buffer(width, height, gl::DEPTH_COMPONENT as i32, gl::DEPTH_COMPONENT, gl::FLOAT, gl::DEPTH_ATTACHMENT);

        // укажем OpenGL, какие буферы мы будем использовать при рендеринге
        let attachments = vec![ gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1, gl::COLOR_ATTACHMENT2, gl::COLOR_ATTACHMENT3, gl::DEPTH_ATTACHMENT ];
        gl::DrawBuffers(3, attachments.as_ptr());
        // После так же добавим буфер глубины и проверку на валидность фреймбуфера.

        (g_buffer, g_position, g_normal, g_color, g_light)
    }
}


//
fn _load_texture_to_gpu<T>(texture: u32, image: &DynamicImage) -> u32 {
    //Бинд текстуры в шейдер
    let mut tex_name = 0u32;
    unsafe {
        gl::GenTextures(1, &mut tex_name);
        gl::ActiveTexture(gl::TEXTURE0 + texture);
        gl::BindTexture( gl::TEXTURE_2D, tex_name );
        gl::TexImage2D ( gl::TEXTURE_2D, 0, gl::RGBA as gl::types::GLint,
                         image.width() as i32, image.height() as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE,
                         image.as_rgba8().unwrap().as_ptr() as *const gl::types::GLvoid);
        gl::TexParameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as gl::types::GLint );
        gl::TexParameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as gl::types::GLint );
    }
    return tex_name;
}

fn load_to_gpu_with_mipmaps(texture: u32, image: &DynamicImage, mipmap_layers: u32, tex_size: (u32, u32)) -> u32 {
    let w = image.width() as i32;
    let h = image.height() as i32;
    let mut tex_name = 0u32;
    unsafe {
        gl::GenTextures(1, &mut tex_name);
        gl::ActiveTexture(gl::TEXTURE0 + texture);
        gl::BindTexture( gl::TEXTURE_2D, tex_name );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, mipmap_layers as i32);

        for i in 0..mipmap_layers {
            let mipmap = game::generate_mipmap(image, tex_size, i as usize);
            let ptr = mipmap.as_rgba8().expect("Cannot cast RGBA8 to pointer").as_ptr();
            gl::TexImage2D ( gl::TEXTURE_2D, i as i32, gl::RGBA as gl::types::GLint, w, h, 0, gl::RGBA, gl::UNSIGNED_BYTE, ptr as *const gl::types::GLvoid);
        }

        gl::TexParameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as gl::types::GLint );
        gl::TexParameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as gl::types::GLint );
        //gl::TexParameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as gl::types::GLint );
    }

    tex_name
}

fn tmp_create_models(max_dist: i32, game: &game::Game) -> (Vec<(game::Chunk, Model)>, Vec<Model>) {
    let mut chunks: Vec<(game::Chunk, Model)> = vec![];
    for cx in 0..max_dist {
        for cy in 0..max_dist {
            let mut chunk = game::Chunk::empty(cx, cy, 0);
            for x in 0..32_i32 {
                for y in 0..32_i32 {
                    let h = ((x + cx * 32).pow(2) + (y + cy*32).pow(2)) as f32;
                    let h = (h * 0.25).sqrt().sin() + 1.0;
                    let h = (h * 2.5).floor() as i32;
                    //let h = 5i32;
                    for z in 0..h {
                        chunk.set_block(3, x, y, z, DenseBools(63));
                    }
                    chunk.set_block(2, x, y, h, DenseBools(63));
                    if x == 10 && y == 10 {
                        for dx in -2..3 {
                            for dy in -2..3 {
                                for dz in -2..3 {
                                    if (dx*dx + dy*dy + dz*dz) <= 2 {
                                        chunk.set_block(6, x + dx, y + dy, h + 8 + dz, DenseBools(63));
                                    }
                                }
                            }
                        }
                        for z in (h+1)..(h + 8) {
                            chunk.set_block(5, x, y, z, DenseBools(0));
                        }
                    }
                }
            }

            if cx == 0 && cy == 0 {
                chunk.set_block(5, 20, 20, 30, DenseBools(0));
            }

            let model = chunk.build_model(game.blocks(), game.models());
            chunks.push((chunk, model));
        }
    }

    let mut blocks: Vec<Model> = vec![];
    for block in game.blocks() {
        let block = {
            let mut vertices: Vec<game::Vertex> = vec![];
            let mut indices: Vec<u32> = vec![];

            let model: &game::BlockModel = &game.models()[block.model_id];
            model.add_to_model(mat::Vec3::new(0.0, 0.0, 0.0), 0, 0, &mut vertices, &mut indices, &block.textures);

            use game::AttribType::*;
            let attributes: Vec<game::AttribType> = vec![Vec3, Vec3, Vec3, Vec3, Vec2, Int, Int];
            game::texture_model(&vertices, &indices, &attributes)
        };
        blocks.push(block);
    }

    (chunks, blocks)
}

fn tmp_display_model() -> Model {
    let max = 1.0;
    let vertices: Vec<f32> = vec![
        -1.0, -1.0, 0.0, 0.0, 0.0,
        -1.0,  max, 0.0, 0.0, 1.0,
         max,  max, 0.0, 1.0, 1.0,
         max, -1.0, 0.0, 1.0, 0.0,
    ];
    let indices: Vec<u32> = vec![
        0, 2, 1,
        0, 3, 2
    ];
    let attribs = vec![game::AttribType::Vec3, game::AttribType::Vec2];

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
                       (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                       vertices.as_ptr() as *const gl::types::GLvoid,
                       gl::STATIC_DRAW,
        );

        let mut offset = 0;
        for (i, attr) in attribs.iter().enumerate() {
            let (numbers_count, data_type) : (i32, gl::types::GLuint) = attr.data_type();
            gl::EnableVertexAttribArray(i as u32);
            gl::VertexAttribPointer( i as u32, numbers_count, data_type,
                                     gl::FALSE, stride,
                                     offset as *const gl::types::GLvoid);
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