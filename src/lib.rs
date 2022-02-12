use image::ImageFormat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::io::{BufRead, Seek};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PixelArt {
    pallet: Vec<u32>,
    buffer: Vec<u32>,
    size: [u32; 2],
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    ImageError(image::ImageError),
    #[error("The length of pallets is longer than 16.")]
    PalletLengthOver16,
}

impl From<image::ImageError> for Error {
    fn from(e: image::ImageError) -> Error {
        Error::ImageError(e)
    }
}

/// pallet display format
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PalletFormat {
    /// U32 decimal integer format, e.g. `11596387`.
    IntegerDecimal,
    /// U32 hexadecimal integer format, e.g. `0xb0f263`
    IntegerHexadecimal,
    /// RGB Integer format, e.g. `176,242,99`
    RGBDecimal,
    /// RGB Integer format, e.g. `0xb0,0xf2,0x63`
    RGBHexadecimal,
    /// RGB Float format, e.g. `0.690,0.949,0.388`
    RGBFloat,
}

impl PalletFormat {
    #[inline]
    pub fn is_integer(&self) -> bool {
        match self {
            PalletFormat::IntegerDecimal => true,
            PalletFormat::IntegerHexadecimal => true,
            _ => false,
        }
    }
}

impl Default for PalletFormat {
    fn default() -> Self {
        Self::RGBDecimal
    }
}

/// buffer display format
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BufferFormat {
    /// Turn the picture upside down so that the index starts at the bottom left of the picture. default: `true`
    pub reverse_rows: bool,
    /// Invert bytes of each chunk. default: `true`
    pub reverse_each_chunk: bool,
    /// Even if the data can be compressed, the buffer will be displayed as an array without compression. default: `false`
    pub force_to_raw: bool,
}

impl Default for BufferFormat {
    fn default() -> Self {
        Self {
            reverse_rows: true,
            reverse_each_chunk: true,
            force_to_raw: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// buffer format
    pub buffer_format: BufferFormat,
    /// pallet format
    pub pallet_format: PalletFormat,
}

#[derive(Clone, Copy, Debug)]
pub struct Display<'a> {
    entity: &'a PixelArt,
    config: DisplayConfig,
}

impl PixelArt {
    /// Creates Bitmap from image file.
    pub fn from_image<R: BufRead + Seek>(file: R, format: ImageFormat) -> Result<PixelArt, Error> {
        let v = image::load(file, format)?;
        let size = [v.width(), v.height()];
        let v = v.into_rgba8().into_raw();
        let mut col2idx = HashMap::new();
        let buffer: Vec<_> = v
            .chunks(4)
            .map(|e| {
                let x = u32::from_be_bytes([0, e[0], e[1], e[2]]);
                let idx = col2idx.len();
                *col2idx.entry(x).or_insert(idx as u32)
            })
            .collect();
        let mut pallet = vec![0; col2idx.len()];
        col2idx
            .into_iter()
            .for_each(|(idx, i)| pallet[i as usize] = idx);
        Ok(PixelArt {
            pallet,
            buffer,
            size,
        })
    }

    #[inline]
    pub fn display(&self, config: DisplayConfig) -> Result<Display, Error> {
        Ok(Display {
            entity: self,
            config,
        })
    }

    /// necessary bit shift for represent pixel
    #[inline]
    pub fn necessary_bit_shift(&self) -> usize {
        usize::pow(
            2,
            f32::ceil(f32::log2(
                1.0 + f32::floor(f32::log2(usize::max(self.pallet.len(), 1) as f32)),
            )) as u32,
        )
    }
    #[inline]
    pub fn is_compressible(&self) -> bool {
        self.pallet.len() < usize::pow(2, 16)
    }
    #[inline]
    pub fn swap_pallet_index(&mut self, i: u32, j: u32) {
        let color = self.pallet[i as usize];
        self.pallet[i as usize] = self.pallet[j as usize];
        self.pallet[j as usize] = color;
        self.buffer.iter_mut().for_each(|idx| {
            if *idx == i {
                *idx = j;
            } else if *idx == j {
                *idx = i;
            }
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct ColorDisplay {
    format: PalletFormat,
    color: u32,
}
impl std::fmt::Display for ColorDisplay {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.format {
            PalletFormat::IntegerDecimal => f.write_fmt(format_args!("{}", self.color)),
            PalletFormat::IntegerHexadecimal => f.write_fmt(format_args!("{:#x}", self.color)),
            PalletFormat::RGBDecimal => f.write_fmt(format_args!(
                "vec3({}, {}, {}) / 255.0",
                (self.color & 0xFF0000) >> 16,
                (self.color & 0x00FF00) >> 8,
                self.color & 0x0000FF
            )),
            PalletFormat::RGBHexadecimal => f.write_fmt(format_args!(
                "vec3({:#x}, {:#x}, {:#x}) / 255.0",
                (self.color & 0xFF0000) >> 16,
                (self.color & 0x00FF00) >> 8,
                self.color & 0x0000FF
            )),
            PalletFormat::RGBFloat => f.write_fmt(format_args!(
                "vec3({:.3}, {:.3}, {:.3})",
                ((self.color & 0xFF0000) >> 16) as f32 / 255.0,
                ((self.color & 0x00FF00) >> 8) as f32 / 255.0,
                (self.color & 0x0000FF) as f32 / 255.0
            )),
        }
    }
}

#[test]
fn pallet_format() {
    let mut display = ColorDisplay {
        format: PalletFormat::IntegerDecimal,
        color: 11596387,
    };
    assert_eq!("11596387", &display.to_string());
    display.format = PalletFormat::IntegerHexadecimal;
    assert_eq!("0xb0f263", &display.to_string());
    display.format = PalletFormat::RGBDecimal;
    assert_eq!("vec3(176, 242, 99) / 255.0", &display.to_string());
    display.format = PalletFormat::RGBHexadecimal;
    assert_eq!("vec3(0xb0, 0xf2, 0x63) / 255.0", &display.to_string());
    display.format = PalletFormat::RGBFloat;
    assert_eq!("vec3(0.690, 0.949, 0.388)", &display.to_string());
}

impl<'a> Display<'a> {
    fn fmt_pallet(&self, f: &mut Formatter) -> std::fmt::Result {
        let format = self.config.pallet_format;
        let output_type = match format {
            PalletFormat::IntegerDecimal => "int",
            PalletFormat::IntegerHexadecimal => "int",
            PalletFormat::RGBDecimal => "vec3",
            PalletFormat::RGBHexadecimal => "vec3",
            PalletFormat::RGBFloat => "vec3",
        };
        f.write_fmt(format_args!(
            "const {output_type} PALLET[] = {output_type}[](\n"
        ))?;
        self.entity
            .pallet
            .iter()
            .copied()
            .enumerate()
            .try_for_each(|(i, color)| {
                let display = ColorDisplay { format, color };
                match i + 1 != self.entity.pallet.len() {
                    true => f.write_fmt(format_args!("    {display},\n")),
                    false => f.write_fmt(format_args!("    {display}\n")),
                }
            })?;
        f.write_str(");\n\n")
    }

    #[inline]
    fn current_row_buffer(&self) -> Vec<u32> {
        match self.config.buffer_format.reverse_rows {
            true => self
                .entity
                .buffer
                .chunks(self.entity.size[0] as usize)
                .rev()
                .flatten()
                .copied()
                .collect(),
            false => self.entity.buffer.clone(),
        }
    }
    fn is_compressible(&self) -> bool {
        !self.config.buffer_format.force_to_raw && self.entity.is_compressible()
    }
    fn compressed_buffer(&self) -> Vec<u32> {
        let buffer = self.current_row_buffer();
        if self.is_compressible() {
            let bit_shift = self.entity.necessary_bit_shift();
            let chunk_size = 32 / bit_shift;
            let closure = move |sum: u32, i: &u32| *i + (sum << bit_shift);
            buffer
                .chunks(chunk_size)
                .map(|a| {
                    let mut a = a.to_vec();
                    a.resize(chunk_size, 0);
                    match self.config.buffer_format.reverse_each_chunk {
                        true => a.iter().rev().fold(0, closure),
                        false => a.iter().fold(0, closure),
                    }
                })
                .collect()
        } else {
            buffer.iter().copied().map(|x| x as u32).collect()
        }
    }
    fn fmt_buffer(&self, f: &mut Formatter) -> Result<bool, std::fmt::Error> {
        let buffer = self.compressed_buffer();
        let format_chunk_size = match self.is_compressible() {
            true => 8,
            false => self.entity.size[0] as usize,
        };
        let intable = buffer.iter().copied().max().unwrap() < 0x800000;
        match intable {
            true => f.write_str("const int BUFFER[] = int[](\n")?,
            false => f.write_str("const uint BUFFER[] = uint[](\n")?,
        }
        buffer
            .chunks(format_chunk_size)
            .enumerate()
            .try_for_each(|(i, x)| {
                f.write_str("    ")?;
                x.iter().enumerate().try_for_each(|(j, px)| {
                    match intable {
                        true => f.write_fmt(format_args!("{}", px))?,
                        false => f.write_fmt(format_args!("{}U", px))?,
                    }
                    match j + 1 == x.len() {
                        true => match i == (buffer.len() - 1) / format_chunk_size {
                            true => f.write_str("\n"),
                            false => f.write_str(",\n"),
                        },
                        false => f.write_str(", "),
                    }
                })?;
                Ok(())
            })?;
        f.write_str(");\n\n")?;
        Ok(intable)
    }
    fn fmt_get_color(&self, intable: bool, f: &mut Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} getColor(in ivec2 u) {{\n",
            match self.config.pallet_format.is_integer() {
                true => "int",
                false => "vec3",
            }
        ))?;
        match self.config.buffer_format.reverse_rows {
            true => f.write_fmt(format_args!(
                "    int idx = u.y * {} + u.x;\n",
                self.entity.size[0]
            ))?,
            false => f.write_fmt(format_args!(
                "    int idx = ({} - u.y) * {} + u.x;\n",
                self.entity.size[1] - 1,
                self.entity.size[0]
            ))?,
        }
        if self.is_compressible() {
            let bit_shift = self.entity.necessary_bit_shift();
            let chunk_size = 32 / bit_shift;
            f.write_fmt(format_args!(
                "    u = ivec2(idx % {chunk_size}, idx / {chunk_size});\n"
            ))?;
            let rem_coef = u32::pow(2, bit_shift as u32) - 1;
            let buffer_suffix = match intable {
                true => "",
                false => "U",
            };
            match self.config.buffer_format.reverse_each_chunk {
                true => f.write_fmt(format_args!(
                    "    return PALLET[BUFFER[u.y] >> u.x * {bit_shift} & {rem_coef}{buffer_suffix}];\n",
                ))?,
                false => f.write_fmt(format_args!(
                    "    return PALLET[BUFFER[u.y] >> ({} - u.x) * {bit_shift} & {rem_coef}{buffer_suffix}];\n",
                    chunk_size - 1,
                ))?,
            }
        } else {
            f.write_fmt(format_args!("    return PALLET[BUFFER[idx]];\n"))?;
        }
        f.write_str("}\n\n")
    }

    fn fmt_main(&self, f: &mut Formatter) -> std::fmt::Result {
        let [width, height] = self.entity.size;
        let get_color = match self.config.pallet_format.is_integer() {
            true => "int2rgb(getColor(u))",
            false => "getColor(u)",
        };
        f.write_fmt(format_args!(
            "void mainImage(out vec4 O, in vec2 U) {{
    vec2 r = iResolution.xy;
    ivec2 u = ivec2(floor((U - 0.5 * r) / r.y * {height}.0 + vec2({:?}, {:?})));
    O = u == abs(u) && u.x < {width} && u.y < {height} ? vec4({get_color}, 1) : vec4(0.5);
}}\n",
            width as f32 / 2.0,
            height as f32 / 2.0,
        ))
    }
}

const INT_TO_RGB: &str = "vec3 int2rgb(int color) {
    return vec3((color & 0xff0000) >> 16, (color & 0xff00) >> 8, color & 0xff) / 255.0;
}\n\n";

impl<'a> std::fmt::Display for Display<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        self.fmt_pallet(f)?;
        let intable = self.fmt_buffer(f)?;
        if self.config.pallet_format.is_integer() {
            f.write_str(INT_TO_RGB)?;
        }
        self.fmt_get_color(intable, f)?;
        self.fmt_main(f)
    }
}

#[test]
fn display_test() {
    let reader = std::io::BufReader::new(std::fs::File::open("resources/heart.png").unwrap());
    let pixel_art = PixelArt::from_image(reader, ImageFormat::Png).unwrap();
    let display = pixel_art
        .display(DisplayConfig {
            buffer_format: BufferFormat {
                //force_to_raw: true,
                ..Default::default()
            },
            pallet_format: PalletFormat::RGBFloat,
        })
        .unwrap();
    println!("{display}");
}
