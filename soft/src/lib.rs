#[macro_use]
mod macros;
pub mod matrix;

pub use soft_macros::Interpolate;

use std::ops::{Add, Mul, Sub};

pub use crate::matrix::*;

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };

    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };
}

impl Default for Color {
    fn default() -> Self {
        Color::BLACK
    }
}

impl From<[f32; 3]> for Color {
    fn from([r, g, b]: [f32; 3]) -> Self {
        Color { r, g, b }
    }
}

impl From<Vector3> for Color {
    fn from(vec: Vector3) -> Self {
        Color {
            r: vec.x,
            g: vec.y,
            b: vec.z,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Triangle<T> {
    pub vertices: [T; 3],
}

impl<T> From<[T; 3]> for Triangle<T> {
    fn from(vertices: [T; 3]) -> Self {
        Triangle { vertices }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PixelCoord {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl From<[u32; 2]> for Dimensions {
    fn from([width, height]: [u32; 2]) -> Self {
        Dimensions { width, height }
    }
}

#[derive(Clone)]
pub struct Texture<T> {
    size: Dimensions,
    pixels: Vec<T>,
}

pub trait PixelBuffer {
    /// Get the size of the buffer
    fn size(&self) -> Dimensions;

    /// Set a pixel to a specific color
    fn set(&mut self, pixel: PixelCoord, color: Color);
}

pub trait Interpolate: Sized {
    fn tri_lerp(values: &[Self; 3], factors: [f32; 3]) -> Self;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum WindingOrder {
    Clockwise,
    CounterClockwise,
}

pub trait ShaderModule {
    type VertexInput;
    type FragmentInput: Interpolate;

    const FRONT_FACE: Option<WindingOrder> = None;

    fn vertex_shader(&self, vertex: &Self::VertexInput) -> (Vector4, Self::FragmentInput);
    fn fragment_shader(&self, fragment: &Self::FragmentInput) -> Color;
}

pub trait VertexBuffer<V> {
    fn get_vertex(&self, index: u32) -> V;
}

type VertexIndex = u32;

pub fn draw<P: PixelBuffer, S: ShaderModule, V: VertexBuffer<S::VertexInput>>(
    pixels: &mut P,
    shaders: &S,
    vertex_buffer: &V,
    indices: &[Triangle<VertexIndex>],
) {
    let mut triangles = Vec::with_capacity(indices.len());
    for triangle in indices {
        let vertex_0 = vertex_buffer.get_vertex(triangle.vertices[0]);
        let vertex_1 = vertex_buffer.get_vertex(triangle.vertices[1]);
        let vertex_2 = vertex_buffer.get_vertex(triangle.vertices[2]);

        let (pos_0, data_0) = shaders.vertex_shader(&vertex_0);
        let (pos_1, data_1) = shaders.vertex_shader(&vertex_1);
        let (pos_2, data_2) = shaders.vertex_shader(&vertex_2);

        let unproject = |vector: Vector4| {
            let inv_w = 1.0 / vector.w;
            Vector4 {
                x: vector.x * inv_w,
                y: vector.y * inv_w,
                z: vector.z * inv_w,
                w: inv_w,
            }
        };

        let positions = [unproject(pos_0), unproject(pos_1), unproject(pos_2)];
        let vertex_datas = [data_0, data_1, data_2];

        triangles.push((positions, vertex_datas));
    }

    let size = pixels.size();
    for (vertices, vertex_data) in triangles {
        let corners = [
            Vector2::from(vertices[0]),
            Vector2::from(vertices[1]),
            Vector2::from(vertices[2]),
        ];

        // cull back faces
        if let Some(front_order) = S::FRONT_FACE {
            if triangle_winding_order(corners) != front_order {
                continue;
            }
        }

        // cull triangles that are obviously outside clip space
        let (min, max) = triangle_bounds(&vertices);
        if max.x < -1.0 || 1.0 < min.x || max.y < -1.0 || 1.0 < min.y || max.z < 0.0 || 1.0 < min.z
        {
            continue;
        }

        let pixel_x_min = ((0.5 + 0.5 * min.x).max(0.0) * size.width as f32).floor() as u32;
        let pixel_x_max = ((0.5 + 0.5 * max.x).min(1.0) * size.width as f32).ceil() as u32;
        let pixel_y_min = ((0.5 - 0.5 * max.y).max(0.0) * size.height as f32).floor() as u32;
        let pixel_y_max = ((0.5 - 0.5 * min.y).min(1.0) * size.height as f32).ceil() as u32;

        for y in pixel_y_min..pixel_y_max {
            for x in pixel_x_min..pixel_x_max {
                let pixel = PixelCoord { x, y };

                let frag_coord = Vector2 {
                    x: 2.0 * (0.5 + pixel.x as f32) / size.width as f32 - 1.0,
                    y: 1.0 - 2.0 * (0.5 + pixel.y as f32) / size.height as f32,
                };

                if let Some(barycentric) = barycentric_coords(corners, frag_coord) {
                    let depth =
                        tri_lerp(&[vertices[0].z, vertices[1].z, vertices[2].z], barycentric);
                    let perspective =
                        tri_lerp(&[vertices[0].w, vertices[1].w, vertices[2].w], barycentric);
                    if 0.0 <= depth && depth <= 1.0 {
                        let inv_perspective = 1.0 / perspective;
                        let interpolation = [
                            barycentric[0] * vertices[0].w * inv_perspective,
                            barycentric[1] * vertices[1].w * inv_perspective,
                            barycentric[2] * vertices[2].w * inv_perspective,
                        ];

                        let frag_data = S::FragmentInput::tri_lerp(&vertex_data, interpolation);
                        let color = shaders.fragment_shader(&frag_data);
                        pixels.set(pixel, color);
                    }
                }
            }
        }
    }
}

fn triangle_bounds(points: &[Vector4; 3]) -> (Vector3, Vector3) {
    let mut min = points[0].truncate();
    let mut max = points[0].truncate();
    for point in points.iter().skip(1) {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        min.z = min.z.min(point.z);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
        max.z = max.z.max(point.z);
    }
    (min, max)
}

fn triangle_winding_order([a, b, c]: [Vector2; 3]) -> WindingOrder {
    let side_area = |current: Vector2, next: Vector2| (next.x - current.x) * (next.y + current.y);
    let areas = [side_area(a, b), side_area(b, c), side_area(c, a)];
    let total_area = areas.iter().sum::<f32>();
    if total_area > 0.0 {
        WindingOrder::Clockwise
    } else {
        WindingOrder::CounterClockwise
    }
}

#[inline(always)]
fn triangle_area([a, b, c]: [Vector2; 3]) -> f32 {
    (a - c).cross(b - c)
}

#[inline(always)]
fn barycentric_coords([a, b, c]: [Vector2; 3], point: Vector2) -> Option<[f32; 3]> {
    let a_area = triangle_area([point, b, c]);
    let b_area = triangle_area([a, point, c]);
    let c_area = triangle_area([a, b, point]);

    let a_sign = a_area.is_sign_positive();
    let b_sign = b_area.is_sign_positive();
    let c_sign = c_area.is_sign_positive();

    if a_sign == b_sign && b_sign == c_sign {
        let inv_area = 1.0 / triangle_area([a, b, c]);
        Some([a_area * inv_area, b_area * inv_area, c_area * inv_area])
    } else {
        None
    }
}

pub fn tri_lerp<T>(values: &[T; 3], factors: [f32; 3]) -> T
where
    T: Add<T, Output = T> + Copy + Mul<f32, Output = T>,
{
    values[0] * factors[0] + values[1] * factors[1] + values[2] * factors[2]
}

impl_elementwise_op!(Color { r, g, b }, Add, add);
impl_elementwise_op!(Color { r, g, b }, Sub, sub);
impl_elementwise_op!(Color { r, g, b }, Mul, mul);
impl_scalar_op!(Color { r, g, b }, Mul<f32>, mul);

impl Interpolate for () {
    fn tri_lerp(_values: &[Self; 3], _factors: [f32; 3]) -> Self {}
}

impl Interpolate for Vector2 {
    fn tri_lerp(values: &[Self; 3], factors: [f32; 3]) -> Self {
        crate::tri_lerp(values, factors)
    }
}

impl Interpolate for Vector3 {
    fn tri_lerp(values: &[Self; 3], factors: [f32; 3]) -> Self {
        crate::tri_lerp(values, factors)
    }
}

impl Interpolate for Vector4 {
    fn tri_lerp(values: &[Self; 3], factors: [f32; 3]) -> Self {
        crate::tri_lerp(values, factors)
    }
}

impl Interpolate for Color {
    fn tri_lerp(values: &[Self; 3], factors: [f32; 3]) -> Self {
        crate::tri_lerp(values, factors)
    }
}

impl<T: Clone> VertexBuffer<T> for Vec<T> {
    fn get_vertex(&self, index: u32) -> T {
        self[index as usize].clone()
    }
}

impl<T> Texture<T> {
    pub fn new(pixels: Vec<T>, size: Dimensions) -> Texture<T> {
        assert_eq!(pixels.len(), size.width as usize * size.height as usize);

        Texture { size, pixels }
    }
}

impl<T: Clone + Default> Texture<T> {
    pub fn sample_nearest(&self, coord: Vector2) -> T {
        if 0.0 <= coord.x && coord.x < 1.0 && 0.0 <= coord.y && coord.y < 1.0 {
            let x = coord.x * (self.size.width) as f32;
            let y = coord.y * (self.size.height) as f32;
            let index = x as usize + y as usize * self.size.width as usize;
            self.pixels[index].clone()
        } else {
            T::default()
        }
    }

    pub fn sample_nearest_clamp(&self, coord: Vector2) -> T {
        let clamped = Vector2 {
            x: coord.x.max(0.0).min(1.0 - f32::EPSILON),
            y: coord.y.max(0.0).min(1.0 - f32::EPSILON),
        };
        self.sample_nearest(clamped)
    }

    pub fn sample_nearest_repeat(&self, coord: Vector2) -> T {
        let repeated = Vector2 {
            x: coord.x - coord.x.floor(),
            y: coord.y - coord.y.floor(),
        };
        self.sample_nearest(repeated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn winding_order() {
        let triangle = [
            Vector2::new(1.0, 2.0),
            Vector2::new(2.0, 2.0),
            Vector2::new(1.0, 1.0),
        ];

        assert_eq!(triangle_winding_order(triangle), WindingOrder::Clockwise);
    }
}
