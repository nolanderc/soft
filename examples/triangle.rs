use minifb::{Key, KeyRepeat, Window};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

struct ImageBuffer {
    size: soft::Dimensions,
    pixels: Vec<Pixel>,
}

type Pixel = u32;

struct Shaders;

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: soft::Vector3,
    pub color: soft::Color,
}

fn main() -> anyhow::Result<()> {
    let mut window = Window::new("triangle", WIDTH as _, HEIGHT as _, Default::default()).unwrap();
    let mut buffer = ImageBuffer::with_size([WIDTH, HEIGHT].into());

    let shaders = Shaders;

    let vertex = |[x, y]: [f32; 2], [r, g, b]: [f32; 3]| Vertex {
        position: soft::Vector3 { x, y, z: 0.0 },
        color: soft::Color { r, g, b },
    };
    let vertices = vec![
        vertex([0.0, 0.5], [1.0, 0.0, 0.0]),
        vertex([0.5, -0.5], [1.0, 1.0, 0.0]),
        vertex([-0.5, -0.5], [1.0, 0.0, 1.0]),
    ];

    let triangles = vec![[0, 1, 2].into()];

    while window.is_open() {
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
        assert!(pixel.x < self.size.width, "pixel out of bounds");
        assert!(pixel.y < self.size.height, "pixel out of bounds");

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
}

impl soft::ShaderModule for Shaders {
    type VertexInput = Vertex;
    type FragmentInput = FragData;

    fn vertex_shader(&self, input: &Self::VertexInput) -> (soft::Vector4, Self::FragmentInput) {
        (input.position.extend(1.0), FragData { color: input.color })
    }

    fn fragment_shader(&self, input: &Self::FragmentInput) -> soft::Color {
        input.color
    }
}
