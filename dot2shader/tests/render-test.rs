#![cfg(feature = "render-test")]

#[macro_use]
extern crate glium;

use dot2shader::*;
use glium::index::PrimitiveType;
#[allow(unused_imports)]
use glium::{glutin, Surface};

fn render(display: &glium::Display, pixels: &[u8], config: DisplayConfig) -> (Vec<u8>, (u32, u32)) {
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
        }
        implement_vertex!(Vertex, position);
        glium::VertexBuffer::new(
            display,
            &[
                Vertex {
                    position: [-1.0, -1.0],
                },
                Vertex {
                    position: [1.0, -1.0],
                },
                Vertex {
                    position: [-1.0, 1.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                },
            ],
        )
        .unwrap()
    };
    let index_buffer = glium::IndexBuffer::new(
        display,
        PrimitiveType::TrianglesList,
        &[0_u16, 2, 1, 1, 2, 3],
    )
    .unwrap();

    let pixel_art = PixelArt::from_image(pixels).unwrap();
    let generated = pixel_art.display(config).unwrap().to_string();
    let frag_shader = match config.inline_level == InlineLevel::Geekest {
        false => {
            "#version 300 es
precision highp float;
uniform vec2 iResolution;
out vec4 outColor;
void mainImage(out vec4, in vec2);
void main() {
    vec4 color;
    mainImage(color, gl_FragCoord.xy);
    outColor = vec4(color.xyz, 1);
}
"
            .to_string()
                + &generated
        }
        true => format!(
            "#version 300 es
precision highp float;
uniform vec2 iResolution;
out vec4 o;
void main() {{
    vec2 r = iResolution.xy;
    vec4 FC = gl_FragCoord;
    o.w = 1.0;

    {generated}
}}
"
        ),
    };
    let program = program!(display,
        300 es => {
            vertex: "#version 300 es
precision highp float;
in vec2 position;
void main() {
    gl_Position = vec4(position, 0, 1);
}
",

            fragment: &frag_shader
        },
    )
    .unwrap();

    let resolution = display.get_framebuffer_dimensions();
    let uniforms = uniform! {
        iResolution: [resolution.0 as f32, resolution.1 as f32],
    };
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target
        .draw(
            &vertex_buffer,
            &index_buffer,
            &program,
            &uniforms,
            &Default::default(),
        )
        .unwrap();
    target.finish().unwrap();
    let image: glium::texture::RawImage2d<'_, u8> = display.read_front_buffer().unwrap();
    (image.data.into_owned(), (image.width, image.height))
}

fn non_geekest_configs() -> impl Iterator<Item = DisplayConfig> {
    [InlineLevel::None, InlineLevel::InlineVariable]
        .iter()
        .copied()
        .flat_map(move |inline_level| {
            [
                PaletteFormat::IntegerDecimal,
                PaletteFormat::IntegerHexadecimal,
                PaletteFormat::RGBDecimal,
                PaletteFormat::RGBHexadecimal,
                PaletteFormat::RGBFloat,
            ]
            .iter()
            .copied()
            .map(move |pallet_format| (inline_level, pallet_format))
        })
        .flat_map(move |(inline_level, palette_format)| {
            [
                BufferFormat {
                    reverse_rows: true,
                    reverse_each_chunk: true,
                    force_to_raw: true,
                },
                BufferFormat {
                    reverse_rows: false,
                    reverse_each_chunk: true,
                    force_to_raw: true,
                },
                BufferFormat {
                    reverse_rows: true,
                    reverse_each_chunk: false,
                    force_to_raw: true,
                },
                BufferFormat {
                    reverse_rows: false,
                    reverse_each_chunk: false,
                    force_to_raw: true,
                },
                BufferFormat {
                    reverse_rows: true,
                    reverse_each_chunk: true,
                    force_to_raw: false,
                },
                BufferFormat {
                    reverse_rows: false,
                    reverse_each_chunk: true,
                    force_to_raw: false,
                },
                BufferFormat {
                    reverse_rows: true,
                    reverse_each_chunk: false,
                    force_to_raw: false,
                },
                BufferFormat {
                    reverse_rows: false,
                    reverse_each_chunk: false,
                    force_to_raw: false,
                },
            ]
            .iter()
            .copied()
            .map(move |buffer_format| DisplayConfig {
                inline_level,
                palette_format,
                buffer_format,
            })
        })
}

fn geekest_configs() -> impl Iterator<Item = DisplayConfig> {
    [
        BufferFormat {
            reverse_rows: true,
            reverse_each_chunk: true,
            force_to_raw: false,
        },
        BufferFormat {
            reverse_rows: false,
            reverse_each_chunk: true,
            force_to_raw: false,
        },
        BufferFormat {
            reverse_rows: true,
            reverse_each_chunk: false,
            force_to_raw: false,
        },
        BufferFormat {
            reverse_rows: false,
            reverse_each_chunk: false,
            force_to_raw: false,
        },
    ]
    .iter()
    .copied()
    .map(move |buffer_format| DisplayConfig {
        inline_level: InlineLevel::Geekest,
        palette_format: PaletteFormat::RGBFloat,
        buffer_format,
    })
}

fn one_render_test(display: &glium::Display, pixels: &[u8], filename: &str, iter: impl Iterator<Item = DisplayConfig>) {
    let mut previous = None;
    iter.for_each(|config| {
        let (vec, (width, height)) = render(&display, pixels, config);
        if let Some(prev) = previous.take() {
            assert_eq!(vec, prev, "different result: {:?}", config);
            previous = Some(vec);
        } else {
            let image = image::ImageBuffer::from_raw(width, height, vec.clone()).unwrap();
            let image = image::DynamicImage::ImageRgba8(image).flipv();
            image.save(filename).unwrap();
            previous = Some(vec);
        }
    })
}

#[test]
fn render_tests() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_visible(true);
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    one_render_test(
        &display,
        include_bytes!("../resources/heart.png"),
        "non-geekest-heart.png",
        non_geekest_configs(),
    );
    one_render_test(
        &display,
        include_bytes!("../resources/steel.png"),
        "non-geekest-steel.png",
        non_geekest_configs(),
    );
    one_render_test(
        &display,
        include_bytes!("../resources/random.png"),
        "non-geekest-random.png",
        non_geekest_configs(),
    );
    one_render_test(
        &display,
        include_bytes!("../resources/heart.png"),
        "geekest-heart.png",
        geekest_configs(),
    );
    one_render_test(
        &display,
        include_bytes!("../resources/steel.png"),
        "geekest-steel.png",
        geekest_configs(),
    );
    one_render_test(
        &display,
        include_bytes!("../resources/random.png"),
        "geekest-random.png",
        geekest_configs(),
    );
}
