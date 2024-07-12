mod fonts;
mod layers;
mod picture;

use fonts::replace_fonts;
use iced::{Application, Settings};
use layers::get_layers;
use picture::{Picture, PictureFlags};
use std::io::Read;
use std::{env, fs, vec};

pub fn main() -> iced::Result {
    let args: Vec<_> = env::args().collect();
    let (svg_content, file_name): (Vec<u8>, String) = if args.len() > 1 {
        let file_name = &args[1];
        let mut file = fs::File::open(file_name).expect("unable to open file");
        let metadata = fs::metadata(&file_name).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        file.read(&mut buffer).expect("buffer overflow");
        (buffer, file_name.to_string())
    } else {
        (Vec::new(), String::new())
    };

    let svg_content = replace_fonts(svg_content);
    let layers = get_layers(&svg_content);

    Picture::run(Settings::with_flags(PictureFlags {
        svg_content,
        file_name,
        layers,
    }))
}
