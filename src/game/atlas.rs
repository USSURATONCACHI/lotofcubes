use std::path::PathBuf;
use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use crate::game::get_first_word;
use crate::mat::Vec4;
use crate::resources::Resources;

pub struct Atlas {
    image: DynamicImage,
    textures: Vec<TexData>,

    atlas_size: (u32, u32),
    texture_size: (u32, u32),
}
pub struct AtlasBuilder {
    names: Vec<String>,
    texture_size: (u32, u32),

    textures: Vec<(String, DynamicImage)>,
    normals: Vec<(String, DynamicImage)>,
    lightmaps: Vec<(String, DynamicImage)>,
}
/** "Паспортные данные" каждой текстуры, собираемой в атлас.
* каждая игровая текстура состоит из энного количества цветовых, нормальных и световых текстур.
* для отображения на конкретном полигоне может использоваться только один из вариантов каждого
* типа одновременно, однако разные полигоны будут случайно выбирать из доступных вариантов
 */
#[derive(Clone, Debug)]
pub struct TexData {
    pub name: String,

    pub textures_count:  usize, //Количество цветовых текстур
    pub normals_count:   usize, //Количество нормальных текстур
    pub lightmaps_count: usize, //Количество световых текстур

    pub tex_id:  u32,  //Айди первого элемента из серии (айди на атласе)
    pub norm_id: u32,
    pub lgmp_id: u32,
}

impl Atlas {
    pub fn image(&self) -> &DynamicImage {
        &self.image
    }
    pub fn textures(&self) -> &Vec<TexData> {
        &self.textures
    }

    pub fn width(&self) -> u32 { self.atlas_size.0 }
    pub fn height(&self) -> u32 { self.atlas_size.1 }
    pub fn size(&self) -> (u32, u32) { self.atlas_size }

    pub fn tex_width(&self) -> u32 { self.texture_size.0 }
    pub fn tex_height(&self) -> u32 { self.texture_size.1 }
    pub fn tex_size(&self) -> (u32, u32) { self.texture_size }
}


impl AtlasBuilder {
    pub fn new(tex_width: u32, tex_height: u32) -> Self {
        AtlasBuilder {
            names: vec!["nil".to_string()],
            texture_size: (tex_width, tex_height),
            textures:  vec![("nil".to_string(), nil_texture(tex_width, tex_height))],
            normals:   vec![("nil".to_string(), nil_normal(tex_width, tex_height))],
            lightmaps: vec![("nil".to_string(), nil_lightmap(tex_width, tex_height))],
        }
    }

    pub fn add_names(&mut self, names: &mut Vec<String>) -> &mut Self {
        self.names.append(names); self
    }
    pub fn load_textures(&mut self, directory: PathBuf, res: &Resources) -> &mut Self {
        let mut textures = load_textures(res, directory, &self.names, self.texture_size);
        self.textures.append(&mut textures);
        self
    }
    pub fn load_normals(&mut self, directory: PathBuf, res: &Resources) -> &mut Self {
        let mut normals = load_textures(res, directory, &self.names, self.texture_size);
        self.normals.append(&mut normals);
        self
    }
    pub fn load_lightmaps(&mut self, directory: PathBuf, res: &Resources) -> &mut Self {
        let mut lightmaps = load_textures(res, directory, &self.names, self.texture_size);
        self.lightmaps.append(&mut lightmaps);
        self
    }

    pub fn build(self) -> Atlas {
        let textures_total = self.textures.len() + self.normals.len() + self.lightmaps.len();
        let tex_w = self.texture_size.0;
        let tex_h = self.texture_size.1;
        //Сколько текстур в размере будет одна сторона атласа
        let atlas_side = (textures_total as f64).sqrt().ceil() as u32;
        let atlas_w = atlas_side * tex_w;
        let atlas_h = atlas_side * tex_h;

        let mut image = DynamicImage::new_rgba8(atlas_w, atlas_h);
        let mut res_textures: Vec<TexData> = Vec::with_capacity(self.names.len());
        let mut last_tex_id = 0u32;

        for n in self.names {
            let mut data = TexData {
                name: n.clone(),
                textures_count: 0, normals_count: 0, lightmaps_count: 0,
                tex_id: last_tex_id, norm_id: 0, lgmp_id: 0,
            };
            //Добавление текстур
            for (tex_name, sprite) in &self.textures {
                if tex_name.eq(&n) {
                    data.textures_count += 1;
                    //Копирование спрайта в атлас (local x, local y)
                    let sprite_x = (last_tex_id % atlas_side) * tex_w;
                    let sprite_y = (last_tex_id / atlas_side) * tex_h;
                    copy_image_to_another(sprite, &mut image, sprite_x, sprite_y);
                    //Переход к следующей текстуре
                    last_tex_id += 1;
                }
            }
            //Добавление нормалей
            data.norm_id = last_tex_id;
            for (tex_name, sprite) in &self.normals {
                if tex_name.eq(&n) {
                    data.normals_count += 1;
                    //Копирование спрайта в атлас (local x, local y)
                    let sprite_x = (last_tex_id % atlas_side) * tex_w;
                    let sprite_y = (last_tex_id / atlas_side) * tex_h;
                    copy_image_to_another(sprite, &mut image, sprite_x, sprite_y);
                    //Переход к следующей текстуре
                    last_tex_id += 1;
                }
            }
            //Добавление карт света
            data.lgmp_id = last_tex_id;
            for (tex_name, sprite) in &self.lightmaps {
                if tex_name.eq(&n) {
                    data.lightmaps_count += 1;
                    //Копирование спрайта в атлас (local x, local y)
                    let sprite_x = (last_tex_id % atlas_side) * tex_w;
                    let sprite_y = (last_tex_id / atlas_side) * tex_h;
                    copy_image_to_another(sprite, &mut image, sprite_x, sprite_y);
                    //Переход к следующей текстуре
                    last_tex_id += 1;
                }
            }

            if data.textures_count == 0 {
                data.textures_count = 1;
                data.tex_id = 0;
            }
            if data.normals_count == 0 {
                data.normals_count = 1;
                data.norm_id = 1;
            }
            if data.lightmaps_count == 0 {
                data.lightmaps_count = 1;
                data.lgmp_id = 2;
            }

            res_textures.push(data);
        }

        Atlas {
            image,
            textures: res_textures,

            atlas_size: (atlas_w, atlas_h),
            texture_size: self.texture_size,
        }
    }
}

/** В names загружается список названий текстур, затем оные компилируются в атлас текстур  */
pub fn load_textures(res: &Resources, dir: PathBuf, names: &Vec<String>, size: (u32, u32)) -> Vec<(String, DynamicImage)> {

    let mut textures_list: Vec<(String, DynamicImage)> = vec![];
    //текстуры, нормали, карты освещения

    let mut images = res.get_all_images_list(dir.clone());
    //Сортировка файлов по алфавиту
    images.sort_by(|a, b| a.to_str().unwrap().to_lowercase().cmp(&b.to_str().unwrap().to_lowercase()));

    for local_path in images {
        //Название первого элемента пути. В "tex.png" это "tex", в "textures/grass/1.png" - "textures"
        let first_word = &get_first_word(local_path.clone())[..];

        for t_name in names {
            if t_name.eq(first_word) {
                let image = res.load_image( dir.join(local_path.clone()) );
                match image {
                    Ok(image) => {
                        if image.width() != size.0 || image.height() != size.1 {
                            println!("Image skipped because of wrong image size ({}x{} required): {}",
                                     size.0, size.1, local_path.clone().to_str().unwrap());
                        } else {
                            //println!("Loaded {} from {}", t_name, local_path.clone().to_str().unwrap());
                            textures_list.push((t_name.clone(), image));
                        }

                    },
                    Err(e) => {
                        println!("Not loaded {} because of {:?} (from {})", t_name, e, local_path.clone().to_str().unwrap());
                    },
                }
            }
        }
    }

    textures_list
}

//Дефолтные текстуры на случай отсутствия обычной
fn nil_texture(w: u32, h: u32) -> DynamicImage {
    let mut image = DynamicImage::new_rgba8(w, h);
    for x in 0..w {
        for y in 0..h {
            //image.put_pixel(x, y, Rgba::from([255u8, 0, 0, 255]));
            if (x + y) % 2 == 0 {
                image.put_pixel(x, y, Rgba::from([47u8, 45, 45, 255]));
            } else {
                image.put_pixel(x, y, Rgba::from([255u8, 51, 51, 255]));
            }
        }
    }
    image
}
fn nil_normal(w: u32, h: u32) -> DynamicImage {
    let mut image = DynamicImage::new_rgba8(w, h);
    for x in 0..w {
        for y in 0..h {
            image.put_pixel(x, y, Rgba::from([128u8, 128, 255, 255]));
        }
    }
    image
}
fn nil_lightmap(w: u32, h: u32) -> DynamicImage {
    let mut image = DynamicImage::new_rgba8(w, h);
    for x in 0..w {
        for y in 0..h {
            image.put_pixel(x, y, Rgba::from([ ((x % 2) * 255) as u8, ((y % 2) * 255) as u8, 0, 255]));
        }
    }
    image
}

fn copy_image_to_another(from: &DynamicImage, to: &mut DynamicImage, x: u32, y: u32) {
    'x: for lx in 0..from.width() {      //local x
        if lx >= to.width() { break 'x; }
        'y: for ly in 0..from.height() { //local y
            if ly >= to.height() { break 'y; }
            to.put_pixel(x + lx, y + ly, from.get_pixel(lx, ly));
            //to.put_pixel(x + lx, y + ly, Rgba::from([255u8, 0, 0, 255]));
        }
    }
}

/** Костыльный метод генерации мипмапов для текстур 15х15, которые нельзя просто поделить на два.
  * Не уменьшает размер изображения, только его сглаживает*/
pub fn generate_mipmap(image: &DynamicImage, tex_size: (u32, u32), level: usize) -> DynamicImage {
    let image_w = image.width() / tex_size.0;
    let image_h = image.height() / tex_size.1;

    let blur_width = 2u32.pow(level as u32);

    let mut res = DynamicImage::new_rgba8(image.width(), image.height());

    for i in 0.. image_w {
        for j in 0..image_h {
            let tx_min = (i * tex_size.0) as f32;   //Начало текстуры
            let ty_min = (j * tex_size.1) as f32;
            let tx_max = ((i + 1) * tex_size.0 - 1) as f32;   //Начало текстуры
            let ty_max = ((j + 1) * tex_size.1 - 1) as f32;

            for pix_x in 0..tex_size.0 {
                for pix_y in 0..tex_size.1 {
                    let mut sum = Vec4::new(0.0, 0.0, 0.0, 0.0);
                    for dx in 0..blur_width {
                        for dy in 0..blur_width {
                            let fdx = dx as f32 - (blur_width as f32 - 1.0)/2.0;
                            let fdy = dy as f32 - (blur_width as f32 - 1.0)/2.0;

                            sum += Vec4::from(get_texel(image,
                                        tx_min + pix_x as f32 + fdx,
                                        ty_min + pix_y as f32 + fdy,
                                        (tx_min, ty_min, tx_max, ty_max)));
                        }
                    }
                    sum /= blur_width.pow(2) as f32;
                    res.put_pixel(tx_min as u32 + pix_x, ty_min as u32 + pix_y, sum.clone().into());
                }
            }

        }
    }

    res
}

/** Пиксель на дробных координатах (линейное сглаживание) */
fn get_texel(image: &DynamicImage, x: f32, y: f32, bd: (f32, f32, f32, f32)) -> Rgba<u8> {
    let bx = x.floor().clamp(bd.0, bd.2);
    let by = y.floor().clamp(bd.1, bd.3);
    let tx = x.ceil().clamp(bd.0, bd.2);
    let ty = y.ceil().clamp(bd.1, bd.3);

    let dx = x.clamp(bd.0, bd.2) - bx;
    let dy = y.clamp(bd.1, bd.3) - by;

    let bb = Vec4::from(image.get_pixel(bx as u32, by as u32));
    let bt = Vec4::from(image.get_pixel(bx as u32, ty as u32));
    let tb = Vec4::from(image.get_pixel(tx as u32, by as u32));
    let tt = Vec4::from(image.get_pixel(tx as u32, ty as u32));

    let bottom = bb * (1.0 - dx) + tb * dx;
    let top    = bt * (1.0 - dx) + tt * dx;

    Rgba::from(bottom * (1.0 - dy) + top * dy)
}