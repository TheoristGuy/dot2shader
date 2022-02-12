use dot2shader::*;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        panic!("usage: dot2shader <input image file> [config json]");
    }
    let path = std::path::Path::new(&args[1]);
    let image_format = match path.extension().map(std::ffi::OsStr::to_str) {
        Some(Some("png")) => image::ImageFormat::Png,
        Some(Some("bmp")) => image::ImageFormat::Bmp,
        Some(Some("gif")) => image::ImageFormat::Gif,
        _ => panic!(""),
    };
    let reader =
        std::io::BufReader::new(std::fs::File::open(path).unwrap_or_else(|e| panic!("{}", e)));
    let pixel_art = PixelArt::from_image(reader, image_format).unwrap_or_else(|e| panic!("{}", e));
    let arg_file = if args.len() > 2 {
        std::fs::read_to_string(&args[2])
            .ok()
            .and_then(|string| serde_json::from_str::<DisplayConfig>(&string).ok())
    } else {
        None
    };
    let default_json = std::fs::read_to_string("default.json")
        .ok()
        .and_then(|string| serde_json::from_str::<DisplayConfig>(&string).ok());
    let config = match (arg_file, default_json) {
        (Some(got), _) => got,
        (None, Some(got)) => got,
        (None, None) => Default::default(),
    };
    let display = pixel_art.display(config).unwrap();
    println!("{display}");
}
