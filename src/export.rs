use resvg::usvg::{Size, Tree};
use resvg::{tiny_skia, usvg};
use std::io::prelude::*;
use std::path::PathBuf;
use std::{fs::File, sync::Arc};

pub(crate) fn generate_svg_tree(svg_content: &[u8]) -> (Tree, Size) {
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();

    let opt = usvg::Options {
        fontdb: Arc::new(fontdb),
        ..Default::default()
    };
    let tree = usvg::Tree::from_data(svg_content, &opt).unwrap();

    let pixmap_size = tree.size();

    (tree, pixmap_size)
}

pub(crate) fn generate_png(tree: &Tree, pixmap_size: &Size, scale: f32) -> Vec<u8> {
    let mut pixmap =
        tiny_skia::Pixmap::new(pixmap_size.width() as u32, pixmap_size.height() as u32).unwrap();
    resvg::render(
        tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );

    pixmap.encode_png().unwrap()
}

pub(crate) fn export_png(png_data: &[u8], svg_name: &String, overwrite: bool) -> Option<()> {
    let mut output_path = PathBuf::from(&svg_name);
    output_path.set_extension("png");
    if output_path.exists() && !overwrite {
        None
    } else {
        let mut file = File::create(output_path).unwrap();
        file.write_all(png_data).unwrap();
        Some(())
    }
}
