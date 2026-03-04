use std::{fs, path::PathBuf};

use png::DeflateCompression;
use resvg::{
    tiny_skia::{Pixmap, PixmapRef},
    usvg::{Options, Size, Transform, Tree},
};

const ICONS_PATH: &str = "assets/icons";
const PROCESSED_ICONS_PATH: &str = "assets/_processed/icons";

fn main() {
    rasterize_icons();
}

fn rasterize_icons() {
    println!("cargo::rerun-if-changed={ICONS_PATH}");

    fs::create_dir_all(PROCESSED_ICONS_PATH).expect("creating output dir should succeed");

    for entry in fs::read_dir(ICONS_PATH).expect("icons path should be readable") {
        let Ok(entry) = entry else {
            eprintln!("Error traversing through icons path: {entry:?}");
            continue;
        };
        rasterize_icon(entry.path());
    }
}

fn rasterize_icon(svg_path: PathBuf) {
    let Some(file_prefix) = svg_path.file_prefix() else {
        eprintln!(
            "Could not figure out the file name of icon at path {}",
            svg_path.display()
        );
        return;
    };

    eprintln!("Processing '{}'", svg_path.display());

    let output_file_name = {
        let mut s = file_prefix.to_os_string();
        s.push(".png");
        s
    };

    let output_path = {
        let mut buf = PathBuf::from(PROCESSED_ICONS_PATH);
        buf.push(output_file_name);
        buf
    };

    eprintln!("...outputting to '{}'", output_path.display());

    let svg_data = fs::read(svg_path).expect("reading svg should succeed");
    let tree = Tree::from_data(
        &svg_data,
        &Options {
            default_size: Size::from_wh(64.0, 64.0).unwrap(),
            ..Default::default()
        },
    )
    .expect("svg should be valid");

    #[expect(clippy::cast_possible_truncation)]
    #[expect(clippy::cast_sign_loss)]
    let mut pixmap = Pixmap::new(tree.size().width() as u32, tree.size().height() as u32)
        .expect("pixmap creation should succeed");

    resvg::render(&tree, Transform::identity(), &mut pixmap.as_mut());

    let png = encode_png_compressed(&pixmap.as_ref()).expect("png encoding should succeed");

    fs::write(output_path, &png).expect("png writing should succeed");
}

fn encode_png_compressed(pixmap: &PixmapRef) -> Result<Vec<u8>, png::EncodingError> {
    let demultiplied_data = pixmap.to_owned().take_demultiplied();

    let mut data = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut data, pixmap.width(), pixmap.height());
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_deflate_compression(DeflateCompression::Level(9));
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&demultiplied_data)?;
    }

    Ok(data)
}
