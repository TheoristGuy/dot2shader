use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Formatter;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
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
#[wasm_bindgen]
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
    #[inline]
    pub fn element_type(&self) -> &'static str {
        match self.is_integer() {
            true => "int",
            false => "vec3",
        }
    }
}

impl Default for PalletFormat {
    fn default() -> Self {
        Self::RGBDecimal
    }
}

/// buffer display format
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BufferFormat {
    /// Turn the picture upside down so that the index starts at the bottom left of the picture. default: `true`
    #[wasm_bindgen(js_name = "reverseRows")]
    pub reverse_rows: bool,
    /// Invert bytes of each chunk. default: `true`
    #[wasm_bindgen(js_name = "reverseEachChunk")]
    pub reverse_each_chunk: bool,
    /// Even if the data can be compressed, the buffer will be displayed as an array without compression. default: `false`
    #[wasm_bindgen(js_name = "forceToRaw")]
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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InlineLevel {
    /// Each value has a meaningful name. There is no magic number.
    None,
    /// The width and height of the image is inlined, and each function is optimized.
    InlineVariable,
    /// Outputs code for Geekest mode. Everything is inlined, there are no line breaks or spaces. Only RGBFloat palette format is supported.
    /// If you copy and paste it as is, it will not work with Shadertoy.
    Geekest,
}

impl Default for InlineLevel {
    fn default() -> Self {
        Self::None
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// buffer format
    #[wasm_bindgen(js_name = "bufferFormat")]
    pub buffer_format: BufferFormat,
    /// pallet format
    #[wasm_bindgen(js_name = "palletFormat")]
    pub pallet_format: PalletFormat,
    /// inline level
    #[wasm_bindgen(js_name = "inlineLevel")]
    pub inline_level: InlineLevel,
}

#[test]
fn default_config() {
    let string = serde_json::to_string_pretty(&DisplayConfig::default()).unwrap();
    std::fs::write("default.json", &string).unwrap();
}

#[derive(Clone, Copy, Debug)]
pub struct Display<'a> {
    entity: &'a PixelArt,
    config: DisplayConfig,
}

impl PixelArt {
    /// Creates Bitmap from image file.
    pub fn from_image(image_buffer: &[u8]) -> Result<PixelArt, Error> {
        let v = image::load_from_memory(image_buffer)?;
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
    fn necessary_bit_shift(&self) -> usize {
        usize::pow(
            2,
            f32::ceil(f32::log2(
                1.0 + f32::floor(f32::log2(usize::max(self.pallet.len() - 1, 1) as f32)),
            )) as u32,
        )
    }
    #[inline]
    fn is_compressible(&self) -> bool {
        self.pallet.len() < usize::pow(2, 16)
    }
}

#[wasm_bindgen]
impl PixelArt {
    #[wasm_bindgen(js_name = "fromImage")]
    pub fn from_image_(buffer: &[u8]) -> Option<PixelArt> {
        PixelArt::from_image(buffer).ok()
    }
    #[wasm_bindgen(js_name = "swapPalletIndex")]
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
    #[wasm_bindgen(js_name = "getCode")]
    pub fn get_code(&self, config: DisplayConfig) -> String {
        Display {
            entity: self,
            config,
        }
        .to_string()
    }
}

#[derive(Clone, Copy, Debug)]
struct ColorDisplay {
    format: PalletFormat,
    space_delim: &'static str,
    color: u32,
}
impl std::fmt::Display for ColorDisplay {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let space = self.space_delim;
        let zero = match self.space_delim.is_empty() {
            true => "",
            false => "0",
        };
        match self.format {
            PalletFormat::IntegerDecimal => f.write_fmt(format_args!("{}", self.color)),
            PalletFormat::IntegerHexadecimal => f.write_fmt(format_args!("{:#x}", self.color)),
            PalletFormat::RGBDecimal => f.write_fmt(format_args!(
                "vec3({},{space}{},{space}{}){space}/{space}255.{zero}",
                (self.color & 0xFF0000) >> 16,
                (self.color & 0x00FF00) >> 8,
                self.color & 0x0000FF
            )),
            PalletFormat::RGBHexadecimal => f.write_fmt(format_args!(
                "vec3({:#x},{space}{:#x},{space}{:#x}){space}/{space}255.{zero}",
                (self.color & 0xFF0000) >> 16,
                (self.color & 0x00FF00) >> 8,
                self.color & 0x0000FF
            )),
            PalletFormat::RGBFloat => {
                let unit = match space == "" {
                    true => 100.0,
                    false => 1000.0,
                };
                let r = (f32::round(((self.color & 0xFF0000) >> 16) as f32 / 255.0 * unit) / unit)
                    .to_string();
                let r = match r.len() > 1 && space == "" {
                    true => &r[1..],
                    false => &r[0..],
                };
                let g = (f32::round(((self.color & 0x00FF00) >> 8) as f32 / 255.0 * unit) / unit)
                    .to_string();
                let g = match g.len() > 1 && space == "" {
                    true => &g[1..],
                    false => &g[0..],
                };
                let b =
                    (f32::round((self.color & 0x0000FF) as f32 / 255.0 * unit) / unit).to_string();
                let b = match b.len() > 1 && space == "" {
                    true => &b[1..],
                    false => &b[0..],
                };
                if r == g && g == b {
                    f.write_fmt(format_args!("vec3({r})"))
                } else {
                    f.write_fmt(format_args!("vec3({r},{space}{g},{space}{b})",))
                }
            }
        }
    }
}

#[test]
fn pallet_format() {
    let mut display = ColorDisplay {
        format: PalletFormat::IntegerDecimal,
        space_delim: " ",
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
    assert_eq!("vec3(0.69, 0.949, 0.388)", &display.to_string());
}

#[derive(Clone, Copy, Debug)]
struct ArrayDisplayConfig {
    return_delim: &'static str,
    indent_delim: &'static str,
    space_delim: &'static str,
    semi_colon: &'static str,
}

impl From<InlineLevel> for ArrayDisplayConfig {
    fn from(e: InlineLevel) -> ArrayDisplayConfig {
        match e {
            InlineLevel::None => ArrayDisplayConfig {
                return_delim: "\n",
                indent_delim: "    ",
                space_delim: " ",
                semi_colon: ";",
            },
            InlineLevel::InlineVariable => ArrayDisplayConfig {
                return_delim: "\n",
                indent_delim: "    ",
                space_delim: " ",
                semi_colon: ";",
            },
            InlineLevel::Geekest => ArrayDisplayConfig {
                return_delim: "",
                indent_delim: "",
                space_delim: "",
                semi_colon: "",
            },
        }
    }
}

impl<'a> Display<'a> {
    fn fmt_pallet_array(&self, f: &mut Formatter) -> std::fmt::Result {
        let format = self.config.pallet_format;
        let output_type = format.element_type();
        let ArrayDisplayConfig {
            return_delim,
            indent_delim,
            space_delim,
            semi_colon,
        } = self.config.inline_level.into();
        f.write_fmt(format_args!("{output_type}[]({return_delim}"))?;
        self.entity
            .pallet
            .iter()
            .copied()
            .enumerate()
            .try_for_each(|(i, color)| {
                let display = ColorDisplay {
                    format,
                    space_delim,
                    color,
                };
                match i + 1 != self.entity.pallet.len() {
                    true => f.write_fmt(format_args!("{indent_delim}{display},{return_delim}")),
                    false => f.write_fmt(format_args!("{indent_delim}{display}{return_delim}")),
                }
            })?;
        f.write_fmt(format_args!("){semi_colon}{return_delim}{return_delim}"))
    }
    fn fmt_non_inline_pallet(&self, f: &mut Formatter) -> std::fmt::Result {
        let output_type = self.config.pallet_format.element_type();
        f.write_fmt(format_args!("const {output_type} PALLET[] = "))?;
        self.fmt_pallet_array(f)
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
    fn compressed_buffer(&self) -> (Vec<u32>, bool) {
        let buffer = self.current_row_buffer();
        let buffer: Vec<u32> = if self.is_compressible() {
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
        };
        let intable = buffer.iter().copied().max().unwrap() < 0x80000000;
        (buffer, intable)
    }
    fn fmt_buffer_array(
        &self,
        buffer: &[u32],
        intable: bool,
        f: &mut Formatter,
    ) -> std::fmt::Result {
        let ArrayDisplayConfig {
            return_delim,
            indent_delim,
            space_delim,
            semi_colon,
        } = self.config.inline_level.into();
        match intable {
            true => f.write_fmt(format_args!("int[]({return_delim}"))?,
            false => f.write_fmt(format_args!("uint[]({return_delim}"))?,
        }
        let format_chunk_size = match self.is_compressible() {
            true => 8,
            false => self.entity.size[0] as usize,
        };
        buffer
            .chunks(format_chunk_size)
            .enumerate()
            .try_for_each(|(i, x)| {
                f.write_fmt(format_args!("{indent_delim}"))?;
                x.iter().enumerate().try_for_each(|(j, px)| {
                    match intable {
                        true => f.write_fmt(format_args!("{}", px))?,
                        false => f.write_fmt(format_args!("{}U", px))?,
                    }
                    match j + 1 == x.len() {
                        true => match i == (buffer.len() - 1) / format_chunk_size {
                            true => f.write_fmt(format_args!("{return_delim}")),
                            false => f.write_fmt(format_args!(",{return_delim}")),
                        },
                        false => f.write_fmt(format_args!(",{space_delim}")),
                    }
                })?;
                Ok(())
            })?;
        f.write_fmt(format_args!("){semi_colon}{return_delim}{return_delim}"))
    }
    fn fmt_non_inline_buffer(&self, f: &mut Formatter) -> Result<bool, std::fmt::Error> {
        let (buffer, intable) = self.compressed_buffer();
        if self.config.inline_level == InlineLevel::None {
            f.write_fmt(format_args!(
                "const int WIDTH = {width}, HEIGHT = {height}",
                width = self.entity.size[0],
                height = self.entity.size[1],
            ))?;
            match self.is_compressible() {
                true => {
                    let bit_shift = self.entity.necessary_bit_shift();
                    let chunk_size = 32 / bit_shift;
                    f.write_fmt(format_args!(", CHUNKS_IN_U32 = {chunk_size};\n"))?
                }
                false => f.write_str(";\n")?,
            }
        }
        match intable {
            true => f.write_str("const int BUFFER[] = ")?,
            false => f.write_str("const uint BUFFER[] = ")?,
        }
        self.fmt_buffer_array(&buffer, intable, f)?;
        Ok(intable)
    }
    fn fmt_get_color(&self, intable: bool, f: &mut Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} getColor(in ivec2 u) {{\n",
            self.config.pallet_format.element_type(),
        ))?;
        let inline_none = self.config.inline_level == InlineLevel::None;
        let width = match inline_none {
            true => "WIDTH".to_string(),
            false => self.entity.size[0].to_string(),
        };
        let semi_height = match inline_none {
            true => "HEIGHT - 1".to_string(),
            false => (self.entity.size[1] - 1).to_string(),
        };
        match self.config.buffer_format.reverse_rows {
            true => f.write_fmt(format_args!("    int idx = u.y * {width} + u.x;\n"))?,
            false => f.write_fmt(format_args!(
                "    int idx = ({semi_height} - u.y) * {width} + u.x;\n"
            ))?,
        }
        if self.is_compressible() {
            let bit_shift = self.entity.necessary_bit_shift();
            let chunks_in_u32 = match inline_none {
                true => "CHUNKS_IN_U32".to_string(),
                false => (32 / bit_shift).to_string(),
            };
            f.write_fmt(format_args!(
                "    u = ivec2(idx % {chunks_in_u32}, idx / {chunks_in_u32});\n"
            ))?;
            if inline_none {
                f.write_str("    int bitShift = 32 / CHUNKS_IN_U32;\n")?;
            }
            let rem_coef = match (inline_none, intable) {
                (true, true) => "(1 << bitShift) - 1".to_string(),
                (true, false) => "(1u << bitShift) - 1u".to_string(),
                (false, true) => format!("{}", (1 << bit_shift) - 1),
                (false, false) => format!("{}U", (1 << bit_shift) - 1),
            };
            let semi_chunks_in_u32 = match inline_none {
                true => "CHUNKS_IN_U32 - 1".to_string(),
                false => (32 / bit_shift - 1).to_string(),
            };
            match self.config.buffer_format.reverse_each_chunk {
                true => f.write_fmt(format_args!(
                    "    return PALLET[BUFFER[u.y] >> u.x * 32 / {chunks_in_u32} & {rem_coef}];\n",
                ))?,
                false => f.write_fmt(format_args!(
                    "    return PALLET[BUFFER[u.y] >> ({semi_chunks_in_u32} - u.x) * 32 / {chunks_in_u32} & {rem_coef}];\n",
                ))?,
            }
        } else {
            f.write_str("    return PALLET[BUFFER[idx]];\n")?;
        }
        f.write_str("}\n\n")
    }
    fn fmt_main(&self, f: &mut Formatter) -> std::fmt::Result {
        let (width, height, float_height, half_vec) =
            match self.config.inline_level == InlineLevel::None {
                true => (
                    "WIDTH".to_string(),
                    "HEIGHT".to_string(),
                    "float(HEIGHT)".to_string(),
                    "vec2(WIDTH, HEIGHT) / 2.0".to_string(),
                ),
                false => (
                    self.entity.size[0].to_string(),
                    self.entity.size[1].to_string(),
                    format!("{}.0", self.entity.size[1]),
                    format!(
                        "vec2({:?}, {:?})",
                        self.entity.size[0] as f32 / 2.0,
                        self.entity.size[1] as f32 / 2.0
                    ),
                ),
            };
        let get_color = match self.config.pallet_format.is_integer() {
            true => "int2rgb(getColor(u))",
            false => "getColor(u)",
        };
        f.write_fmt(format_args!(
            "void mainImage(out vec4 O, in vec2 U) {{
    vec2 r = iResolution.xy;
    ivec2 u = ivec2(floor((U - 0.5 * r) / r.y * {float_height} + {half_vec}));
    O.xyz = u == abs(u) && u.x < {width} && u.y < {height} ? {get_color} : vec3(0.5);
}}\n"
        ))
    }
    fn fmt_geekest(&self, f: &mut Formatter) -> std::fmt::Result {
        let [width, height] = self.entity.size;
        let size_vec = match width == height {
            true => format!("{}.", width),
            false => format!("vec2({},{})", width, height),
        };
        f.write_fmt(format_args!("ivec2 u=ivec2(FC.xy/r*{size_vec});"))?;
        if self.is_compressible() {
            let bit_shift = self.entity.necessary_bit_shift();
            let chunks_in_u32 = 32 / bit_shift;
            if width != chunks_in_u32 as u32 {
                f.write_fmt(format_args!(
                    "int idx=u.y*{width}+u.x;u=ivec2(idx%{chunks_in_u32},idx/{chunks_in_u32});"
                ))?;
            }
        }
        f.write_str("o.xyz=")?;
        self.fmt_pallet_array(f)?;
        f.write_str("[")?;
        let (buffer, intable) = self.compressed_buffer();
        self.fmt_buffer_array(&buffer, intable, f)?;
        match self.is_compressible() {
            true => {
                let bit_shift = self.entity.necessary_bit_shift();
                let rem_coef = (1 << bit_shift) - 1;
                f.write_fmt(format_args!("[u.y]>>u.x*{bit_shift}&{rem_coef}"))?
            }
            false => f.write_str("[u.y*{width}+u.x]")?,
        }
        f.write_str("];")?;
        Ok(())
    }
}

const INT_TO_RGB: &str = "vec3 int2rgb(int color) {
    return vec3((color & 0xff0000) >> 16, (color & 0xff00) >> 8, color & 0xff) / 255.0;
}\n\n";

impl<'a> std::fmt::Display for Display<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if self.config.inline_level == InlineLevel::Geekest {
            self.fmt_geekest(f)
        } else {
            self.fmt_non_inline_pallet(f)?;
            let intable = self.fmt_non_inline_buffer(f)?;
            if self.config.pallet_format.is_integer() {
                f.write_str(INT_TO_RGB)?;
            }
            self.fmt_get_color(intable, f)?;
            self.fmt_main(f)
        }
    }
}
