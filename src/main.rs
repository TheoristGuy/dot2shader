use dot2shader::*;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        panic!("usage: dot2shader-cli <input image file> [config json]");
    }
    let path = std::path::Path::new(&args[1]);
    let buffer = std::fs::read(&path).unwrap_or_else(|e| panic!("{}", e));
    let pixel_art = PixelArt::from_image(&buffer).unwrap_or_else(|e| panic!("{}", e));
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
