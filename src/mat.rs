/*
Это модуль (библиотека) реализующая все нужные мне функции матричной и векторной математики.
Я написал ее вручную, а не скачал готовую лишь потому, что мне было интересно написать свою.
*/

use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};
use image::Rgba;

//Функционал матриц 4х4
#[derive(Copy, Clone)]
/** Матрица содержит массив 4х4, заполняется построчно, т.е. первые четыре элемента - первая строка.
 Вторые четыре - вторая и т.д. Схематичное изображение номеров элементов массива:
 Mat4
|  0  1  2  3 |
|  4  5  6  7 |
|  8  9 10 11 |
| 12 13 14 15 |
]*/
pub struct Mat4 (pub [f32; 16]);

/** Базовый функционал матриц*/
impl Mat4 {
    /** Единичная матрица, умноженная на число */
    pub fn new(det: f32) -> Mat4 {
        let mut res = [0.0_f32; 16];
        for i in 0..4 {
            res[i * 5] = det;
        }
        Mat4(res)
    }

    /** Заполнить матрицу числом */
    pub fn filled(with: f32) -> Mat4 {
        Mat4([with; 16])
    }

    /** Транспонирование матрицы */
    pub fn transpos(&self) -> Mat4 {
        let mut res: [f32; 16] = [0.0; 16];
        for i in 0..4 {
            for j in 0..4 {
                res[i * 4 + j] = self.0[j * 4 + i];
            }
        }
        Mat4(res)
    }

    pub fn get(&self, i: usize, j: usize) -> f32 {
        self.0[i * 4 + j]
    }
    pub fn set(&mut self, i: usize, j: usize, val: f32) {
        self.0[i * 4 + j] = val;
    }

    //Вычеркнуть строку и столбец
    pub fn sub_matrix(&self, row: usize, col: usize) -> [f32; 9] {
        let mut res = [0_f32; 9];

        let mut tmp_i: usize = 0;
        let mut tmp_j: usize;

        for i in 0..4 {
            tmp_j = 0;
            if i != row {
                for j in 0..4 {
                    if j != col {
                        res[tmp_i * 3 + tmp_j] = self.0[i * 4 + j];
                        tmp_j += 1;
                    }
                }
                tmp_i += 1;
            }
        }

        res
    }

    //Определитель
    pub fn det(&self) -> f32 {
        self.0[0] * sub_det(self.sub_matrix(0, 0)) -
        self.0[1] * sub_det(self.sub_matrix(0, 1)) +
        self.0[2] * sub_det(self.sub_matrix(0, 2)) -
        self.0[3] * sub_det(self.sub_matrix(0, 3))
    }

    //Алгебраические дополнения
    pub fn alg_add(&self) -> Mat4 {
        let mut res = Mat4::new(0.0);
        for i in 0..4 {
            for j in 0..4 {
                res.0[i * 4 + j] = (-1.0_f32).powf((i + j) as f32) * sub_det(self.sub_matrix(i, j));
            }
        }
        res
    }

    //Обратная матрица
    pub fn inverse(&self) -> Mat4 {
        self.alg_add().transpos() / self.det()
    }
}
/** Набор конструкторов для полезных матриц, вроде матриц поворота, сдвига и т.д.*/
impl Mat4 {
    //Матрица сдвига, поворота, масштаба по трем осям
    pub fn object_mat(dx: f32, dy: f32, dz: f32, ax: f32, ay: f32, az: f32, sx: f32, sy: f32, sz: f32) -> Mat4 {
        /*Mat4([
            ay.cos() * az.cos()    * sx,
            -ay.cos() * az.sin()      * sy,
            ay.sin()               * sz,
            dx,

            (ax.sin() * ay.sin() * az.cos() + ax.cos() * az.sin())   * sx,
            (-ax.sin() * ay.sin() * az.sin() + ax.cos() * az.cos())   * sy,
            -ax.sin() * ay.cos()                                    * sz,
            dy,

            (-ax.cos() * ay.sin() * az.cos() + ax.sin() * az.sin())   * sx,
            (ax.cos() * ay.sin() * az.sin() + ax.sin() * az.cos())   * sy,
            ax.cos() * ay.cos()                                    * sz,
            dz,

            0.0,
            0.0,
            0.0,
            1.0,
        ])*/
        Mat4::translate_mat(dx, dy, dz) * Mat4::rotation_mat(ax, ay, az) * Mat4::scale_mat(sx, sy, sz)

    }

    pub fn scale_mat(sx: f32, sy: f32, sz: f32) -> Mat4 {
        Mat4([
            sx, 0.0, 0.0, 0.0,
            0.0, sy, 0.0, 0.0,
            0.0, 0.0, sz, 0.0,
            0.0, 0.0, 0.0, 1.0
        ])
    }

    //Матрица сдвига
    pub fn translate_mat(dx: f32, dy: f32, dz: f32) -> Mat4 {
        Mat4([
            1.0, 0.0, 0.0, dx,
            0.0, 1.0, 0.0, dy,
            0.0, 0.0, 1.0, dz,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    //Матрица поворота по X
    pub fn rot_x_mat(ax: f32) -> Mat4 {
        Mat4([
            1.0, 0.0,       0.0,        0.0,
            0.0, ax.cos(),  -ax.sin(),  0.0,
            0.0, ax.sin(),  ax.cos(),   0.0,
            0.0, 0.0,       0.0,        1.0,
        ])
    }
    //Матрица поворота по Y
    pub fn rot_y_mat(ay: f32) -> Mat4 {
        Mat4([
            ay.cos(),   0.0,  ay.sin(),  0.0,
            0.0,        1.0,  0.0,       0.0,
            -ay.sin(),  0.0,  ay.cos(),  0.0,
            0.0,        0.0,  0.0,       1.0,
        ])
    }
    //Матрица поворота по Z
    pub fn rot_z_mat(az: f32) -> Mat4 {
        Mat4([
            az.cos(),  -az.sin(),  0.0,  0.0,
            az.sin(),   az.cos(),  0.0,  0.0,
            0.0,       0.0,        1.0,  0.0,
            0.0,       0.0,        0.0,  1.0,
        ])
    }

    //Матрица поворота по трем осям
    pub fn rotation_mat(ax: f32, ay: f32, az: f32) -> Mat4 {
        /*Mat4::rot_z_mat(az) * Mat4::rot_y_mat(ay) * Mat4::rot_x_mat(ax)*/
        Mat4([
            az.cos() * ay.cos(),
            az.cos() * ay.sin() * az.sin() - az.sin() * ax.cos(),
            az.sin() * ax.sin() + az.cos() * ay.sin() * ax.cos(),
            0.0,

            az.sin() * ay.cos(),
            az.cos() * ax.cos() + az.sin() * ay.sin() * ax.sin(),
            az.sin() * ay.sin() * ax.cos() - az.cos() * ax.sin(),
            0.0,

            -ay.sin(),
            ay.cos() * ax.sin(),
            ay.cos() * ax.cos(),
            0.0,

            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }

    //Матрица камеры (Right, Up, Forward, Position)
    pub fn view_mat(rx: f32, ry: f32, rz: f32,
                    ux: f32, uy: f32, uz: f32,
                    fx: f32, fy: f32, fz: f32,
                    x: f32, y: f32, z: f32) -> Mat4 {
        Mat4([rx, ux, fx, -x,
            ry, uy, fy, -y,
            rz, uz, fz, -z,
            0.0, 0.0, 0.0, 1.0]).transpos()
    }

    pub fn default_view_mat() -> Mat4 {
        Mat4([1.0, 0.0, 0.0, 0.0,
              0.0, 0.0, 1.0, 0.0,
              0.0, 1.0, 0.0, 0.0,
              0.0, 0.0, 0.0, 1.0])
    }

    //Камера, сдвинутая в точку, затем повернутая в плоскости yz на vert_ang (при том, что y - вперед), затем в xy на horz_ang
    pub fn cam_mat(vert_ang: f32, horz_ang: f32, x: f32, y: f32, z: f32) -> Mat4 {
        /*Mat4([
            horz_ang.cos(),                     -horz_ang.sin(),                    0.0,                -x * horz_ang.cos() + y * horz_ang.sin(),
            vert_ang.sin() * horz_ang.sin(),    vert_ang.sin() * horz_ang.cos(),    vert_ang.cos(),     -x * vert_ang.sin() * horz_ang.sin() - y * vert_ang.sin() * horz_ang.cos() - z * vert_ang.cos(),
            vert_ang.cos() * horz_ang.sin(),    vert_ang.cos() * horz_ang.cos(),    -vert_ang.sin(),    -x * vert_ang.cos() * horz_ang.sin() - y * vert_ang.cos() * horz_ang.cos() + z * vert_ang.sin(),
            0.0,                                0.0,                                0.0,                1.0,
        ])*/
        Mat4::view_mat(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0) *
        Mat4::rot_x_mat(-vert_ang) *
        Mat4::rot_z_mat(horz_ang) *
        Mat4::translate_mat(-x, -y, -z)
    }

    //Матрица перспективы
    pub fn perspective_mat(fov_y: f32, aspect: f32, z_near: f32, z_far: f32) -> Mat4 {
        let q = 1.0 / (fov_y / 2.0).tan();
        let a = q / aspect;
        let b = (z_near + z_far) / (z_near - z_far);
        let c = (2.0 * z_near * z_far) / (z_near - z_far);

        Mat4([
            a, 0.0, 0.0, 0.0,
            0.0, q, 0.0, 0.0,
            0.0, 0.0, b, c,
            0.0, 0.0, -1.0, 0.0,
        ])
    }

    //Матрица ортографической проекции
    pub fn orthographic_mat(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
        Mat4([
            2.0 / (right - left), 0.0, 0.0, -(right + left) / (right - left),
            0.0, 2.0 / (top - bottom), 0.0, -(top + bottom) / (top - bottom),
            0.0, 0.0, 2.0 / (far - near),  -(far + near) / (far - near),
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    /** Матрица вращения вокруг произвольной оси axis */
    pub fn axis_rotation_mat( axis: &Vec4, angle: f32) -> Mat4 {
        let (ang_y, ang_z) = axis.get_yz_angles();

        Mat4::rotation_mat(0.0, ang_y, ang_z) *
        Mat4::rot_x_mat(angle) *
        Mat4::rot_y_mat(-ang_y) *
        Mat4::rot_z_mat(-ang_z)
    }
}

//Определитель фрагмента 3х3
pub fn sub_det(m: [f32; 9]) -> f32 {
    m[0] * m[4] * m[8] +
        m[2] * m[3] * m[7] +
        m[1] * m[5] * m[6] -

        m[2] * m[4] * m[6] -
        m[0] * m[5] * m[7] -
        m[1] * m[3] * m[8]
    /*
    0 1 2
    3 4 5
    6 7 8
    */
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vec4  { Vec4 { x,       y,       z,       w }       }
    pub fn from_slice(s: [f32; 4]) -> Vec4              { Vec4 { x: s[0], y: s[1], z: s[2], w: s[3] } }

    pub fn len(&self) -> f32 {
        (self.x.powf(2.0) +
         self.y.powf(2.0) +
         self.z.powf(2.0) +
         self.w.powf(2.0) ).sqrt()
    }

    /** Единичный вектор, совпадающий направлением с данным*/
    pub fn unit(&self) -> Self {
        *self / self.len()
    }

    /** Возвращает два угла: повернув вектор {1; 0; 0} вокруг оси Y на первый угол,
        затем вокруг оси Z на второй угол - получится единичный вектор направления,
        идентичного  оригинальному
        Mat4::rotation_mat(0.0, vec.get_yz_angles().0, vec.get_yz_angles().1) == vec / vec.len() */
    pub fn get_yz_angles(&self) -> (f32, f32) {
        Vec3::new(self.x, self.y, self.z).get_yz_angles()
    }

    pub fn x(&self) -> f32 { self.x }
    pub fn y(&self) -> f32 { self.y }
    pub fn z(&self) -> f32 { self.z }
    pub fn w(&self) -> f32 { self.w }
}


#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3  {
        Vec3 { x,       y,       z }
    }
    pub fn from_slice(s: [f32; 3]) -> Vec3 {
        Vec3 { x: s[0], y: s[1], z: s[2] }
    }

    pub fn len(&self) -> f32 {
        (self.x.powf(2.0) +
         self.y.powf(2.0) +
         self.z.powf(2.0)  ).sqrt()
    }

    /** Возвращает два угла: повернув вектор {1; 0; 0} вокруг оси Y на первый угол,
           затем вокруг оси Z на второй угол - получится единичный вектор направления,
           идентичного  оригинальному
           Mat4::rotation_mat(0.0, vec.get_yz_angles().0, vec.get_yz_angles().1) == vec / vec.len() */
    pub fn get_yz_angles(&self) -> (f32, f32) {
        let vec = *self / self.len(); //Единичный вектор
        let xy_len = (vec.x().powf(2.0) + vec.y().powf(2.0)).sqrt(); //Длина проекции на плоскость xy
        //println!("-====---= Self: {:?}, len: {}, Vec: {:?}, XY len: {}", self, self.len(), vec, xy_len);

        let ang_y = ( xy_len ).acos();
        if xy_len == 0.0 {
            return ( if vec.z() >= 0.0 { -ang_y } else { ang_y }, 0.0)
        }

        let ang_z = ( vec.x() / xy_len ).acos();
        (
            if vec.z() >= 0.0 { -ang_y } else { ang_y },
            if vec.y() >= 0.0 { ang_z } else { -ang_z }
        )
    }

    pub fn x(&self) -> f32 { self.x }
    pub fn y(&self) -> f32 { self.y }
    pub fn z(&self) -> f32 { self.z }
}



/* Куча скучных трейтов математических действий */


//Матрица + Матрица
impl std::ops::Add<Mat4> for Mat4 {
    type Output = Mat4;

    fn add(self, rhs: Mat4) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = self.0[i] + rhs.0[i]; }
        new_mat
    }
}
//Матрица + Число
impl std::ops::Add<f32> for Mat4 {
    type Output = Mat4;

    fn add(self, rhs: f32) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = self.0[i] + rhs; }
        new_mat
    }
}
//Число + Матрица
impl std::ops::Add<Mat4> for f32 {
    type Output = Mat4;

    fn add(self, rhs: Mat4) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = rhs.0[i] + self; }
        new_mat
    }
}

//Матрица - Матрица
impl std::ops::Sub<Mat4> for Mat4 {
    type Output = Mat4;

    fn sub(self, rhs: Mat4) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = self.0[i] - rhs.0[i]; }
        new_mat
    }
}
//Матрица - Число
impl std::ops::Sub<f32> for Mat4 {
    type Output = Mat4;

    fn sub(self, rhs: f32) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = self.0[i] - rhs; }
        new_mat
    }
}
//Число - Матрица
impl std::ops::Sub<Mat4> for f32 {
    type Output = Mat4;

    fn sub(self, rhs: Mat4) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = self - rhs.0[i]; }
        new_mat
    }
}

//Матрица * Матрица
impl std::ops::Mul<Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..4 {
            for j in 0..4 {
                let id = i * 4 + j;
                for k in 0..4 {
                    new_mat.0[id] += self.0[i * 4 + k] * rhs.0[k * 4 + j];
                }
            }
        }
        new_mat
    }
}
//Матрица * Число
impl std::ops::Mul<f32> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: f32) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = self.0[i] * rhs; }
        new_mat
    }
}
//Число * Матрица
impl std::ops::Mul<Mat4> for f32 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = rhs.0[i] * self; }
        new_mat
    }
}


//Матрица / Матрица
impl std::ops::Div<Mat4> for Mat4 {
    type Output = Mat4;

    fn div(self, rhs: Mat4) -> Mat4 {
        self * rhs.inverse()
    }
}
//Матрица / Число
impl std::ops::Div<f32> for Mat4 {
    type Output = Mat4;

    fn div(self, rhs: f32) -> Mat4 {
        let mut new_mat: Mat4 = Mat4::filled(0.0);
        for i in 0..16 { new_mat.0[i] = self.0[i] / rhs; }
        new_mat
    }
}
//Число / Матрица
impl std::ops::Div<Mat4> for f32 {
    type Output = Mat4;

    fn div(self, rhs: Mat4) -> Mat4 {
        self * rhs.inverse()
    }
}

//Вывод матрицы
impl std::fmt::Debug for Mat4 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut max_width = [0usize; 4];
        for row in 0..4 {
            for col in 0..4 {
                let id = 4 * row + col;
                let str_len = self.0[id].to_string().len();
                if str_len > max_width[col] { max_width[col] = str_len; }
            }
        };

        write!(f, "Mat4:\n")?;
        for row in 0..4 {
            write!(f, "| ")?;
            for col in 0..4 {
                let id = 4 * row + col;
                let str_len = self.0[id].to_string().len();
                write!(f, "{}", &self.0[id])?;
                write!(f, "{}", " ".chars().cycle().take(max_width[col] - str_len + 1).collect::<String>())?;
            }
            write!(f, "|\n")?;
        };
        Ok(())
    }
}


/* ======= Vec4 ======== */
//Вектор + Вектор
impl std::ops::Add<Vec4> for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self.x() + rhs.x(), self.y() + rhs.y(), self.z() + rhs.z(), self.w() + rhs.w())
    }
}
//Вектор += Вектор
impl AddAssign for Vec4 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
//Вектор + Число
impl std::ops::Add<f32> for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: f32) -> Vec4 {
        Vec4::new(self.x() + rhs, self.y() + rhs, self.z() + rhs, self.w() + rhs)
    }
}
//Вектор += Число
impl AddAssign<f32> for Vec4 {
    fn add_assign(&mut self, other: f32) {
        *self = *self + other;
    }
}
//Число + Вектор
impl std::ops::Add<Vec4> for f32 {
    type Output = Vec4;
    fn add(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self + rhs.x(), self + rhs.y(), self + rhs.z(), self + rhs.w())
    }
}

//Вектор - Вектор
impl std::ops::Sub<Vec4> for Vec4 {
    type Output = Vec4;
    fn sub(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self.x() - rhs.x(), self.y() - rhs.y(), self.z() - rhs.z(), self.w() - rhs.w())
    }
}
//Вектор -= Вектор
impl SubAssign for Vec4 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}
//Вектор - Число
impl std::ops::Sub<f32> for Vec4 {
    type Output = Vec4;
    fn sub(self, rhs: f32) -> Vec4 {
        Vec4::new(self.x() - rhs, self.y() - rhs, self.z() - rhs, self.w() - rhs)
    }
}
//Вектор -= Число
impl SubAssign<f32> for Vec4 {
    fn sub_assign(&mut self, other: f32) {
        *self = *self - other;
    }
}
//Число - Вектор
impl std::ops::Sub<Vec4> for f32 {
    type Output = Vec4;
    fn sub(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self - rhs.x(), self - rhs.y(), self - rhs.z(), self - rhs.w())
    }
}

//Вектор * Вектор
impl std::ops::Mul<Vec4> for Vec4 {
    type Output = f32;
    fn mul(self, rhs: Vec4) -> f32 {
        self.x() * rhs.x() + self.y() * rhs.y() + self.z() * rhs.z() + self.w() * rhs.w()
    }
}
//Вектор * Число
impl std::ops::Mul<f32> for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: f32) -> Vec4 {
        Vec4::new(self.x() * rhs, self.y() * rhs, self.z() * rhs, self.w() * rhs)
    }
}
//Вектор *= Число
impl MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, other: f32) {
        *self = *self * other;
    }
}
//Число * Вектор
impl std::ops::Mul<Vec4> for f32 {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self * rhs.x(), self * rhs.y(), self * rhs.z(), self * rhs.w())
    }
}
//Матрица * Вектор
impl std::ops::Mul<Vec4> for Mat4 {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4::new(
            self.get(0, 0) * rhs.x() + self.get(0, 1) * rhs.y() + self.get(0, 2) * rhs.z() + self.get(0, 3) * rhs.w(),
            self.get(1, 0) * rhs.x() + self.get(1, 1) * rhs.y() + self.get(1, 2) * rhs.z() + self.get(1, 3) * rhs.w(),
            self.get(2, 0) * rhs.x() + self.get(2, 1) * rhs.y() + self.get(2, 2) * rhs.z() + self.get(2, 3) * rhs.w(),
            self.get(3, 0) * rhs.x() + self.get(3, 1) * rhs.y() + self.get(3, 2) * rhs.z() + self.get(3, 3) * rhs.w())
    }
}

//Вектор / Число
impl std::ops::Div<f32> for Vec4 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self::new(
            self.x() / rhs,
            self.y() / rhs,
            self.z() / rhs,
            self.w() / rhs,
        )
    }
}
//Вектор /= Число
impl DivAssign<f32> for Vec4 {
    fn div_assign(&mut self, other: f32) {
        *self = *self / other;
    }
}


/* ======= Vec3 ======== */
//Вектор + Вектор
impl std::ops::Add<Vec3> for Vec3 {
    type Output = Self;
    fn add(self, rhs: Vec3) -> Self {
        Self::new(self.x() + rhs.x(), self.y() + rhs.y(), self.z() + rhs.z())
    }
}
//Вектор += Вектор
impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
//Вектор + Число
impl std::ops::Add<f32> for Vec3 {
    type Output = Self;
    fn add(self, rhs: f32) -> Self {
        Self::new(self.x() + rhs, self.y() + rhs, self.z() + rhs)
    }
}
//Вектор += Число
impl AddAssign<f32> for Vec3 {
    fn add_assign(&mut self, other: f32) {
        *self = *self + other;
    }
}
//Число + Вектор
impl std::ops::Add<Vec3> for f32 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self + rhs.x(), self + rhs.y(), self + rhs.z())
    }
}

//Вектор - Вектор
impl std::ops::Sub<Vec3> for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x() - rhs.x(), self.y() - rhs.y(), self.z() - rhs.z())
    }
}
//Вектор -= Вектор
impl SubAssign for Vec3 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}
//Вектор - Число
impl std::ops::Sub<f32> for Vec3 {
    type Output = Self;
    fn sub(self, rhs: f32) -> Self {
        Self::new(self.x() - rhs, self.y() - rhs, self.z() - rhs)
    }
}
//Вектор -= Число
impl SubAssign<f32> for Vec3 {
    fn sub_assign(&mut self, other: f32) {
        *self = *self - other;
    }
}
//Число - Вектор
impl std::ops::Sub<Vec3> for f32 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self - rhs.x(), self - rhs.y(), self - rhs.z())
    }
}

//Вектор * Вектор
impl std::ops::Mul<Vec3> for Vec3 {
    type Output = f32;
    fn mul(self, rhs: Vec3) -> f32 {
        self.x() * rhs.x() + self.y() * rhs.y() + self.z() * rhs.z()
    }
}
//Вектор * Число
impl std::ops::Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x() * rhs, self.y() * rhs, self.z() * rhs)
    }
}
//Вектор *= Число
impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, other: f32) {
        *self = *self * other;
    }
}
//Число * Вектор
impl std::ops::Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self * rhs.x(), self * rhs.y(), self * rhs.z())
    }
}

//Вектор / Число
impl std::ops::Div<f32> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self::new(
            self.x() / rhs,
            self.y() / rhs,
            self.z() / rhs,
        )
    }
}
//Вектор /= Число
impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, other: f32) {
        *self = *self / other;
    }
}

impl From<Vec4> for Vec3 {
    fn from(item: Vec4) -> Self {
        Self::new(item.x, item.y, item.z)
    }
}
impl From<Vec3> for Vec4 {
    fn from(item: Vec3) -> Self {
        Self::new(item.x, item.y, item.z, 0.0)
    }
}

impl From<Rgba<u8>> for Vec4 {
    fn from(item: Rgba<u8>) -> Self {
        Self::new(item.0[0] as f32, item.0[1] as f32, item.0[2] as f32, item.0[3] as f32)
    }
}
impl From<Vec4> for Rgba<u8> {
    fn from(item: Vec4) -> Self {
        Rgba::from([item.x().round() as u8, item.y().round() as u8, item.z().round() as u8, item.w().round() as u8])
    }
}