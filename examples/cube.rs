use std::time::Instant;

use minifb::{Key, KeyRepeat, Window};

use soft::matrix::*;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

struct ImageBuffer {
    size: soft::Dimensions,
    pixels: Vec<Pixel>,
}

type Pixel = u32;

struct Shaders {
    time: f32,
    transformation: Matrix4,
    texture: soft::Texture<soft::Color>,
}

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: soft::Vector3,
    pub color: soft::Color,
    pub tex_coord: soft::Vector2,
}

fn main() -> anyhow::Result<()> {
    let mut window = Window::new("cube", WIDTH as _, HEIGHT as _, Default::default()).unwrap();
    let mut buffer = ImageBuffer::with_size([WIDTH, HEIGHT].into());

    let mut shaders = Shaders {
        time: 0.0,
        transformation: Matrix4::identity(),
        texture: soft::Texture::new(
            vec![
                [0.2; 3].into(),
                [1.0; 3].into(),
                [1.0; 3].into(),
                [0.2; 3].into(),
            ],
            [2, 2].into(),
        ),
    };

    let vertex = |position: [f32; 3], color: [f32; 3], tex_coord: [f32; 2]| Vertex {
        position: position.into(),
        color: color.into(),
        tex_coord: tex_coord.into(),
    };
    let vertices = vec![
        // front
        vertex([-0.5, -0.5, -0.5], [0.0, 0.0, 0.0], [0.0, 0.0]),
        vertex([-0.5, 00.5, -0.5], [0.0, 1.0, 0.0], [0.0, 1.0]),
        vertex([00.5, 00.5, -0.5], [1.0, 1.0, 0.0], [1.0, 1.0]),
        vertex([00.5, -0.5, -0.5], [1.0, 0.0, 0.0], [1.0, 0.0]),
        // back
        vertex([-0.5, -0.5, 00.5], [0.0, 0.0, 1.0], [1.0, 0.0]),
        vertex([-0.5, 00.5, 00.5], [0.0, 1.0, 1.0], [1.0, 1.0]),
        vertex([00.5, 00.5, 00.5], [1.0, 1.0, 1.0], [0.0, 1.0]),
        vertex([00.5, -0.5, 00.5], [1.0, 0.0, 1.0], [0.0, 0.0]),
        // right
        vertex([00.5, -0.5, -0.5], [0.0, 0.0, 0.0], [0.0, 0.0]),
        vertex([00.5, 00.5, -0.5], [0.0, 1.0, 0.0], [0.0, 1.0]),
        vertex([00.5, 00.5, 00.5], [1.0, 1.0, 0.0], [1.0, 1.0]),
        vertex([00.5, -0.5, 00.5], [1.0, 0.0, 0.0], [1.0, 0.0]),
        // left
        vertex([-0.5, -0.5, -0.5], [0.0, 0.0, 0.0], [1.0, 0.0]),
        vertex([-0.5, 00.5, -0.5], [0.0, 1.0, 0.0], [1.0, 1.0]),
        vertex([-0.5, 00.5, 00.5], [1.0, 1.0, 0.0], [0.0, 1.0]),
        vertex([-0.5, -0.5, 00.5], [1.0, 0.0, 0.0], [0.0, 0.0]),
        // top
        vertex([-0.5, 00.5, -0.5], [0.0, 0.0, 0.0], [0.0, 0.0]),
        vertex([00.5, 00.5, -0.5], [0.0, 1.0, 0.0], [1.0, 0.0]),
        vertex([00.5, 00.5, 00.5], [1.0, 1.0, 0.0], [1.0, 1.0]),
        vertex([-0.5, 00.5, 00.5], [1.0, 0.0, 0.0], [0.0, 1.0]),
        // bottom
        vertex([-0.5, -0.5, -0.5], [0.0, 0.0, 0.0], [0.0, 1.0]),
        vertex([00.5, -0.5, -0.5], [0.0, 1.0, 0.0], [1.0, 1.0]),
        vertex([00.5, -0.5, 00.5], [1.0, 1.0, 0.0], [1.0, 0.0]),
        vertex([-0.5, -0.5, 00.5], [1.0, 0.0, 0.0], [0.0, 0.0]),
    ];

    let triangles = vec![
        // front
        [0, 3, 2].into(),
        [2, 1, 0].into(),
        // back
        [4, 5, 6].into(),
        [6, 7, 4].into(),
        // right
        [8, 11, 10].into(),
        [10, 9, 8].into(),
        // left
        [12, 13, 14].into(),
        [14, 15, 12].into(),
        // top
        [16, 17, 18].into(),
        [18, 19, 16].into(),
        // bottom
        [20, 23, 22].into(),
        [22, 21, 20].into(),
    ];

    let mut previous_frame = Instant::now();
    let mut frames = 0;
    let mut elapsed = 0.0;
    while window.is_open() {
        let now = Instant::now();
        let dt = now.saturating_duration_since(previous_frame).as_secs_f32();
        previous_frame = now;
        frames += 1;
        elapsed += dt;
        if elapsed > 0.5 {
            let fps = frames as f32 / elapsed;
            frames = 0;
            elapsed = 0.0;
            window.set_title(&format!("cube @ {} fps", fps.round()));
        }

        shaders.time += dt;

        shaders.transformation = {
            let projection = Matrix4::from(Perspective {
                fov: Deg(70.0).into(),
                aspect: WIDTH as f32 / HEIGHT as f32,
                near: 0.1,
                far: 10.0,
            });
            let view = Matrix4::translate(Vector3::new(0.0, 0.0, 2.0));
            let model = Matrix4::rotate(Rad(0.5 * (-0.5 * shaders.time).sin()), Vector3::unit_x())
                * Matrix4::rotate(Rad(shaders.time), Vector3::unit_y());

            projection * view * model
        };

        buffer.pixels.iter_mut().for_each(|pixel| *pixel = 0x303030);
        soft::draw(&mut buffer, &shaders, &vertices, &triangles);
        window
            .update_with_buffer(
                &buffer.pixels,
                buffer.size.width as _,
                buffer.size.height as _,
            )
            .unwrap();
        if window.is_key_pressed(Key::Escape, KeyRepeat::No) {
            break;
        }
    }

    Ok(())
}

impl ImageBuffer {
    pub fn with_size(size: soft::Dimensions) -> ImageBuffer {
        ImageBuffer {
            size,
            pixels: vec![0; (size.width * size.height) as usize],
        }
    }
}

impl soft::PixelBuffer for ImageBuffer {
    fn size(&self) -> soft::Dimensions {
        self.size
    }

    fn set(&mut self, pixel: soft::PixelCoord, color: soft::Color) {
        debug_assert!(pixel.x < self.size.width, "pixel out of bounds");
        debug_assert!(pixel.y < self.size.height, "pixel out of bounds");

        let index = pixel.x + pixel.y * self.size.width;
        self.pixels[index as usize] = pixel_from_color(color);
    }
}

fn pixel_from_color(color: soft::Color) -> Pixel {
    let r = (255.0 * color.r) as u8 as u32;
    let g = (255.0 * color.g) as u8 as u32;
    let b = (255.0 * color.b) as u8 as u32;
    (r << 16) | (g << 8) | b
}

#[derive(soft::Interpolate)]
struct FragData {
    color: soft::Color,
    tex_coord: Vector2,
}

impl soft::ShaderModule for Shaders {
    type VertexInput = Vertex;
    type FragmentInput = FragData;

    const FRONT_FACE: Option<soft::WindingOrder> = Some(soft::WindingOrder::CounterClockwise);

    fn vertex_shader(&self, input: &Self::VertexInput) -> (soft::Vector4, Self::FragmentInput) {
        let position = self.transformation * input.position.extend(1.0);
        (
            position,
            FragData {
                color: input.tex_coord.extend(0.0).into(),
                tex_coord: input.tex_coord,
            },
        )
    }

    fn fragment_shader(&self, input: &Self::FragmentInput) -> soft::Color {
        self.texture.sample_nearest_repeat(2.5 * input.tex_coord) * input.color
    }
}
