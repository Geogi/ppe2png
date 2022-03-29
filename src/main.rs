use std::{
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use embedded_graphics::{pixelcolor::RgbColor, prelude::Size};
use ignore::{types::TypesBuilder, DirEntry, WalkBuilder, WalkState};
use png::{BitDepth, ColorType, Compression, Encoder, FilterType};

fn main() -> Result<()> {
    fs::create_dir("converted")?;
    WalkBuilder::new(".")
        .types({
            let mut types_builder = TypesBuilder::new();
            types_builder.add("bmp", "*.bmp")?;
            types_builder.select("bmp").build()?
        })
        .build_parallel()
        .run(|| {
            Box::new(|maybe_de| match par_work(maybe_de) {
                Ok(()) => WalkState::Continue,
                Err(err) => {
                    println!("{:?}", err);
                    WalkState::Quit
                }
            })
        });
    Ok(())
}

fn par_work(maybe_de: Result<DirEntry, ignore::Error>) -> Result<()> {
    let dir_entry = maybe_de?;
    if dir_entry.file_type().ok_or(anyhow!("no type"))?.is_file() {
        let in_file = fs::read(dir_entry.path())?;
        let bmp = tinybmp::Bmp::<embedded_graphics::pixelcolor::Rgb888>::from_slice(&in_file)
            .map_err(|_| anyhow!("couldn’t parse"))?;
        let Size { width, height } = bmp.as_raw().size();
        let mut data = vec![];
        for p in bmp.pixels() {
            data.push(p.1.r());
            data.push(p.1.g());
            data.push(p.1.b());
        }
        let mut p = PathBuf::from(".");
        p.push("converted");
        p.push(Path::new(dir_entry.file_name()).with_extension("png"));
        let w = BufWriter::new(File::create(p)?);
        let mut encoder = Encoder::new(w, width, height);
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);
        encoder.set_compression(Compression::Rle);
        encoder.set_filter(FilterType::Paeth);
        //encoder.set_adaptive_filter(png::AdaptiveFilterType::Adaptive);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&data)?;
    }
    Ok(())
}
