use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read};
use std::ffi;
use image::DynamicImage;
use crate::glsl_expand::{ExpandError, ShaderContext};
use crate::resources::Error::{CouldNotDecodeImage, CouldNotLoadImage};


#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    FileContainsNil,
    FailedToGetExePath,
    CouldNotLoadImage(String),
    CouldNotDecodeImage(image::error::ImageError),
}
impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self { Error::Io(other) }
}

pub struct Resources {
    root_path: PathBuf,
    shader_context: ShaderContext,
}
impl Resources {
    pub fn from_relative(rel_path: &Path) -> Result<Resources, Error> {
        let exe_file_name = ::std::env::current_exe().map_err(|_| Error::FailedToGetExePath)?;
        let exe_path = exe_file_name.parent().ok_or(Error::FailedToGetExePath)?;

        let root_path = exe_path.join(rel_path);
        Ok(
            Resources {
                root_path: root_path.clone(),
                shader_context: ShaderContext::from_dir(root_path.clone()).unwrap(),
            }
        )
    }
    pub fn load_cstring(&self, resource_name: &str) -> Result<ffi::CString, Error> {
        let mut file = fs::File::open( self.root_path.join(resource_name) )?;

        // allocate buffer of the same size as file
        let mut buffer: Vec<u8> = Vec::with_capacity( file.metadata()?.len() as usize + 1 );
        file.read_to_end(&mut buffer)?;

        // check for nul byte
        if buffer.iter().find(|i| **i == 0).is_some() {
            return Err(Error::FileContainsNil);
        }

        Ok(unsafe { ffi::CString::from_vec_unchecked(buffer) })
    }
    pub fn load_png(&self, resource_name: &str) -> Result<DynamicImage, Error> {
        let path = self.root_path.join(resource_name);
        match image::io::Reader::open(path.clone()) {
            Err(_) => {
                Err(CouldNotLoadImage(path.to_str().unwrap().to_owned()))
            },
            Ok(image) => {
                 match image.decode() {
                     Ok(result) => { Ok(result) }
                     Err(e) => { Err(CouldNotDecodeImage(e)) }
                 }
            }
        }
    }

    /** Возвращает набор локальных путей и самих изображений (всех в директории и поддиректориях) */
    pub fn load_all_images(&self, dir: &str) -> Vec<(PathBuf, DynamicImage)> {
        let main_path = self.root_path.join(dir);
        let mut files_list: Vec<(PathBuf, FileFormat)> = Vec::new();
        read_directory_to_vec(&main_path, &mut files_list);

        let mut result: Vec<(PathBuf, DynamicImage)> = Vec::new();

        for (file_path, file_format) in files_list {
            match file_format {
                FileFormat::Png => {
                    match image::io::Reader::open(file_path.clone()) {
                        Err(_) => { println!("Failed to load image {:?}", file_path); }
                        Ok(image) => {
                            match image.decode() {
                                Ok(image) => {
                                    let local_path = file_path.strip_prefix(main_path.clone()).unwrap();
                                    result.push((local_path.to_path_buf(), image ) );
                                }
                                Err(e) => {
                                    println!("Failed to decode image {:?}: {:?}", file_path, e);
                                }
                            }
                        }
                    }
                }
                _ => {}
            };
        }

        result
    }

    /** Возвращает список всех изображений (локальный путь) в папке, всех подпапках и т.д.*/
    pub fn get_all_images_list(&self, dir: PathBuf) -> Vec<PathBuf> {
        let main_path = self.root_path.join(dir);
        let mut files_list: Vec<(PathBuf, FileFormat)> = Vec::new();
        read_directory_to_vec(&main_path, &mut files_list);

        let mut result: Vec<PathBuf> = Vec::new();

        for (file_path, file_format) in files_list {
            match file_format {
                FileFormat::Png => {
                    let local_path = file_path.strip_prefix(main_path.clone()).unwrap();
                    result.push(local_path.to_path_buf() );
                }
                _ => {}
            };
        }

        result
    }

    /** Пытается загрузить изображение по полученному пути */
    pub fn load_image(&self, local_path: PathBuf) -> Result<DynamicImage, Error> {
        let path = self.root_path.join(local_path);
        match image::io::Reader::open(path.clone()) {
            Err(_) => {
                Err(CouldNotLoadImage(path.to_str().unwrap().to_owned()))
            },
            Ok(image) => {
                match image.decode() {
                    Ok(result) => { Ok(result) }
                    Err(e) => { Err(CouldNotDecodeImage(e)) }
                }
            }
        }
    }

    pub fn load_shader_text(&mut self, path: PathBuf) -> Result<&String, ExpandError> {
        Ok(self.shader_context.get_file_processed(path)?.current_text())
    }

    pub fn get_absolute_path(&self, local_path: PathBuf) -> PathBuf {
        self.root_path.join(local_path)
    }
}

/** Форматы файлов (которые я использую) */
#[derive(Debug, PartialEq)]
pub enum FileFormat {
    Folder,
    Png,

    Frag,
    Vert,

    Other,
    Error,
}
impl FileFormat {
    pub fn from_string(s: String) -> FileFormat {
        match &s.to_lowercase()[..] {
            "" => FileFormat::Folder,
            "png" => FileFormat::Png,
            "frag" => FileFormat::Frag,
            "vert" => FileFormat::Vert,
            _ => FileFormat::Other,
        }
    }
    pub fn from_path_buf(path: &PathBuf) -> FileFormat {
        let filename = path.file_name();
        if let None = filename { return FileFormat::Error; }
        let split: Vec<String> = filename.unwrap().to_str().unwrap().split(".").map(|x| x.to_owned()).collect();
        if split.len() == 1 {
            FileFormat::Folder
        } else {
            FileFormat::from_string(split[split.len() - 1].clone())
        }
    }
    pub fn is_shader(&self) -> bool {
        match self {
            Self::Frag => true,
            Self::Vert => true,
            _ => false,
        }
    }
    fn _is_image(&self) -> bool {
        match self {
            Self::Png => { true }
            _ => { false }
        }
    }
}

fn _resource_name_to_path(root_dir: &Path, location: &str) -> PathBuf {
    let mut path: PathBuf = root_dir.into();
    for part in location.split("/") { path = path.join(part); }
    path
}
/** Читает все доступные файлы в директории и поддиректориях, добавляет их в вектор по ссылке */
pub fn read_directory_to_vec(dir: &PathBuf, vec: &mut Vec<(PathBuf, FileFormat)>) {
    let format = FileFormat::from_path_buf(dir);      //Проверка на директорию
    if format != FileFormat::Folder { return; }
    let paths = fs::read_dir(dir);

    if let Err(_) = paths { return; }            //Обработка ошибки
    let paths = paths.unwrap();

    for path in paths {
        if let Err(_) = path { continue; } //Отсеивание ошибочных файлов
        let path = path.unwrap().path();
        let format = FileFormat::from_path_buf(&path);

        match format {
            FileFormat::Other => {}
            FileFormat::Error => {}

            FileFormat::Folder => { read_directory_to_vec(&path, vec); }
            f => { vec.push((path, f)); }
        };
    }
}