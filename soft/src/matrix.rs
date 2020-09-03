use std::ops::{Add, Index, IndexMut, Mul, Sub};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Matrix2 {
    pub x: Vector2,
    pub y: Vector2,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Matrix3 {
    pub x: Vector3,
    pub y: Vector3,
    pub z: Vector3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Matrix4 {
    pub x: Vector4,
    pub y: Vector4,
    pub z: Vector4,
    pub w: Vector4,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Perspective {
    pub fov: Rad,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

/// An angle expressed in radians
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rad(pub f32);

/// An angle expressed in degrees
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Deg(pub f32);

macro_rules! intersperse {
    ($separator:tt, [$head:tt, $($tail:tt),*]) => {
        $head $($separator $tail)*
    };
}

macro_rules! count {
    (@replace: $tt:tt => $value:literal) => { $value };
    ($($tt:tt),*) => { 0usize $(+ count!(@replace: $tt => 1usize))* };
}

macro_rules! impl_vector {
    ($vector:ident {$($field:ident),*}) => {
        impl_elementwise_op!($vector { $($field),* }, Add, add);
        impl_elementwise_op!($vector { $($field),* }, Sub, sub);
        impl_scalar_op!($vector { $($field),* }, Mul<f32>, mul);

        impl $vector {
            pub const LENGTH: usize = count!($($field),*);
            pub const ORIGIN: $vector = $vector {
                $(
                    $field: 0.0,
                )*
            };

            pub const fn new($($field: f32),*) -> Self {
                Self { $($field),* }
            }

            #[inline(always)]
            pub fn dot(self, other: Self) -> f32 {
                intersperse!(
                    +,
                    [$((self.$field * other.$field)),*]
                )
            }

            #[inline(always)]
            pub fn length2(self) -> f32 {
                Self::dot(self, self)
            }

            #[inline(always)]
            pub fn length(self) -> f32 {
                self.length2().sqrt()
            }

            pub fn normalized(self) -> $vector {
                self.length().recip() * self
            }

            #[inline(always)]
            pub fn as_arr(&self) -> &[f32; Self::LENGTH] {
                unsafe {
                    &*(self as *const $vector as *const [f32; Self::LENGTH])
                }
            }

            #[inline(always)]
            pub fn as_arr_mut(&mut self) -> &mut [f32; Self::LENGTH] {
                unsafe {
                    &mut *(self as *mut $vector as *mut [f32; Self::LENGTH])
                }
            }

            #[inline(always)]
            pub fn to_arr(self) -> [f32; Self::LENGTH] {
                unsafe {
                    std::mem::transmute(self)
                }
            }
        }

        impl Default for $vector {
            fn default() -> Self {
                Self {
                    $(
                        $field: 0.0,
                    )*
                }
            }
        }

        impl From<[f32; Self::LENGTH]> for $vector {
            fn from([$($field),*]: [f32; Self::LENGTH]) -> Self {
                Self {
                    $($field),*
                }
            }
        }

        impl Index<usize> for $vector {
            type Output = f32;
            #[inline(always)]
            fn index(&self, index: usize) -> &Self::Output {
                &self.as_arr()[index]
            }
        }

        impl IndexMut<usize> for $vector {
            #[inline(always)]
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                &mut self.as_arr_mut()[index]
            }
        }
    }
}

macro_rules! impl_matrix {
    ($matrix:ident [$vector:ident { $($field:ident),* }]) => {
        impl $matrix {
            pub const SIZE: usize = count!($($field),*);

            #[inline(always)]
            pub const fn from_rows([$($field),*]: [$vector; Self::SIZE]) -> Self {
                Self { $( $field, )* }
            }

            #[inline(always)]
            pub fn from_cols([$($field),*]: [$vector; Self::SIZE]) -> Self {
                Self { $( $field, )* }.transpose()
            }

            pub fn transpose(mut self) -> Self {
                let arr = self.as_arr_mut();

                for row in 1..Self::SIZE {
                    for col in 0..row {
                        arr.swap(col + row * Self::SIZE, row + col * Self::SIZE)
                    }
                }

                self
            }

            #[inline(always)]
            pub fn as_arr_mut(&mut self) -> &mut [f32; Self::SIZE * Self::SIZE] {
                unsafe {
                    &mut *(self as *mut $matrix as *mut _)
                }
            }
        }

        impl From<[[f32; Self::SIZE]; Self::SIZE]> for $matrix {
            fn from([$($field),*]: [[f32; Self::SIZE]; Self::SIZE]) -> Self {
                Self {
                    $(
                        $field: $vector::from($field),
                    )*
                }
            }
        }

        impl From<[$vector; Self::SIZE]> for $matrix {
            fn from([$($field),*]: [$vector; Self::SIZE]) -> Self {
                Self {
                    $( $field, )*
                }
            }
        }

        impl Mul<$vector> for $matrix {
            type Output = $vector;
            #[inline]
            fn mul(self, rhs: $vector) -> Self::Output {
                $vector {
                    $(
                        $field: $vector::dot(self.$field, rhs),
                    )*
                }
            }
        }

        impl Mul<$matrix> for $matrix {
            type Output = $matrix;
            #[inline]
            fn mul(self, rhs: $matrix) -> Self::Output {
                let rhs_t = rhs.transpose();
                $matrix {
                    $(
                        $field: rhs_t * self.$field,
                    )*
                }
            }
        }
    }
}

impl_vector!(Vector2 { x, y });
impl_vector!(Vector3 { x, y, z });
impl_vector!(Vector4 { x, y, z, w });

impl_matrix!(Matrix2[Vector2 { x, y }]);
impl_matrix!(Matrix3[Vector3 { x, y, z }]);
impl_matrix!(Matrix4[Vector4 { x, y, z, w }]);

macro_rules! vector_unit_axis {
    ($vector:ident { $($field:ident = $fn:ident),* }) => {
        impl $vector {
            $(
                pub const fn $fn() -> $vector {
                    $vector {
                        $field: 1.0,
                        ..Self::ORIGIN
                    }
                }
            )*
        }
    }
}

vector_unit_axis!(Vector2 { x = unit_x, y = unit_y });
vector_unit_axis!(Vector3 { x = unit_x, y = unit_y, z = unit_z });
vector_unit_axis!(Vector4 { x = unit_x, y = unit_y, z = unit_z, w = unit_w });

impl Vector2 {
    #[inline(always)]
    pub fn extend(self, z: f32) -> Vector3 {
        Vector3 {
            x: self.x,
            y: self.y,
            z,
        }
    }

    #[inline(always)]
    pub fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }
}

impl Vector3 {
    #[inline(always)]
    pub fn extend(self, w: f32) -> Vector4 {
        Vector4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w,
        }
    }

    #[inline(always)]
    pub fn truncate(self) -> Vector2 {
        self.into()
    }
}

impl Vector4 {
    #[inline(always)]
    pub fn truncate(self) -> Vector3 {
        Vector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl From<Vector4> for Vector3 {
    fn from(vec: Vector4) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
        }
    }
}

impl From<Vector3> for Vector2 {
    fn from(vec: Vector3) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
        }
    }
}

impl From<Vector4> for Vector2 {
    fn from(vec: Vector4) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
        }
    }
}

impl Matrix2 {
    pub const fn identity() -> Matrix2 {
        Matrix2::from_rows([Vector2::new(1.0, 0.0), Vector2::new(0.0, 1.0)])
    }

    #[inline(always)]
    pub fn rotate(angle: Rad) -> Matrix2 {
        let (sin, cos) = angle.0.sin_cos();
        Matrix2::from([[cos, -sin], [sin, cos]])
    }
}

impl Matrix3 {
    pub const fn identity() -> Matrix3 {
        Matrix3::from_rows([
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ])
    }

    pub fn rotate(angle: Rad, axis: Vector3) -> Matrix3 {
        let (sin, cos) = angle.0.sin_cos();
        let inv_cos = 1.0 - cos;

        let Vector3 { x, y, z } = axis;

        // https://en.wikipedia.org/wiki/Rotation_matrix#In_three_dimensions
        Matrix3::from([
            [
                cos + x * x * inv_cos,
                x * y * inv_cos - z * sin,
                x * z * inv_cos + y * sin,
            ],
            [
                y * x * inv_cos + z * sin,
                cos + y * y * inv_cos,
                y * z * inv_cos - x * sin,
            ],
            [
                z * x * inv_cos - y * sin,
                z * y * inv_cos + x * sin,
                cos + z * z * inv_cos,
            ],
        ])
    }

    pub fn scale(scale: f32) -> Matrix3 {
        Matrix3::from([[scale, 0.0, 0.0], [0.0, scale, 0.0], [0.0, 0.0, scale]])
    }
}

impl Matrix4 {
    pub const fn identity() -> Matrix4 {
        Matrix4::from_rows([
            Vector4::new(1.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, 1.0, 0.0, 0.0),
            Vector4::new(0.0, 0.0, 1.0, 0.0),
            Vector4::new(0.0, 0.0, 0.0, 1.0),
        ])
    }

    pub fn translate(amount: Vector3) -> Matrix4 {
        Matrix4::from([
            [1.0, 0.0, 0.0, amount.x],
            [0.0, 1.0, 0.0, amount.y],
            [0.0, 0.0, 1.0, amount.z],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn scale(scale: f32) -> Matrix4 {
        Matrix4::from([
            [scale, 0.0, 0.0, 0.0],
            [0.0, scale, 0.0, 0.0],
            [0.0, 0.0, scale, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotate(angle: Rad, axis: Vector3) -> Matrix4 {
        Matrix3::rotate(angle, axis).into()
    }
}

impl From<Matrix3> for Matrix4 {
    #[inline]
    fn from(mat3: Matrix3) -> Self {
        Matrix4 {
            x: mat3.x.extend(0.0),
            y: mat3.y.extend(0.0),
            z: mat3.z.extend(0.0),
            w: [0.0, 0.0, 0.0, 1.0].into(),
        }
    }
}

impl From<Perspective> for Matrix4 {
    #[inline]
    fn from(perspective: Perspective) -> Self {
        let Perspective {
            fov,
            aspect,
            near,
            far,
        } = perspective;

        let top = (fov.0 / 2.0).tan();
        let right = aspect * top;
        let depth = far - near;

        Matrix4::from([
            [1.0 / top, 0.0, 0.0, 0.0],
            [0.0, 1.0 / right, 0.0, 0.0],
            [0.0, 0.0, (far + near) / depth, -far * near / depth],
            [0.0, 0.0, 1.0, 0.0],
        ])
    }
}

impl From<Deg> for Rad {
    #[inline(always)]
    fn from(Deg(angle): Deg) -> Self {
        Rad(angle.to_radians())
    }
}

impl From<Rad> for Deg {
    #[inline(always)]
    fn from(Rad(angle): Rad) -> Self {
        Deg(angle.to_degrees())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transpose_matrix() {
        assert_eq!(
            Matrix3::from([[0.0, 1.0, 2.0], [3.0, 4.0, 5.0], [6.0, 7.0, 8.0]]).transpose(),
            Matrix3::from([[0.0, 3.0, 6.0], [1.0, 4.0, 7.0], [2.0, 5.0, 8.0],])
        )
    }

    #[test]
    fn matrix_vector_multiplication() {
        assert_eq!(
            Matrix2::from([[1.0, 2.0], [3.0, 4.0]]) * Vector2::from([5.0, 6.0]),
            Vector2::from([5.0 * 1.0 + 6.0 * 2.0, 5.0 * 3.0 + 6.0 * 4.0])
        )
    }

    #[test]
    fn matrix_multiplication() {
        assert_eq!(
            Matrix2::from([[1.0, 2.0], [3.0, 4.0]]) * Matrix2::from([[5.0, 6.0], [7.0, 8.0]]),
            Matrix2::from([
                [1.0 * 5.0 + 2.0 * 7.0, 1.0 * 6.0 + 2.0 * 8.0],
                [3.0 * 5.0 + 4.0 * 7.0, 3.0 * 6.0 + 4.0 * 8.0],
            ])
        )
    }
}
