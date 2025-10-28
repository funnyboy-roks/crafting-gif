#![allow(clippy::identity_op)]

use std::{fs::File, io::BufWriter, path::Path};

use gif::{Encoder, Frame};
use image::{GenericImageView, ImageBuffer, ImageReader, Pixel, Rgb};

/// Size of a slot in `crafting_table.png`
const GRID_SIZE: u32 = 16;

/// The amount that `crafting_table.png` should be upscaled
const RATIO: u32 = 5;

fn grid_position(grid_position: u32) -> (u32, u32) {
    let icon_size = GRID_SIZE * RATIO;

    let (cx, cy) = if grid_position == 9 {
        // corner of output slot is (93, 17)
        // Output slot is 24x24
        ((93 + 13) * RATIO, (17 + 13) * RATIO)
    } else {
        let left = 4 + GRID_SIZE / 2;
        let top = 4 + GRID_SIZE / 2;
        let pad = GRID_SIZE + 2;

        let x = grid_position % 3;
        let y = grid_position / 3;

        let cx = (left + pad * x) * RATIO;
        let cy = (top + pad * y) * RATIO;
        (cx, cy)
    };
    (cx - (icon_size / 2), cy - (icon_size / 2))
}

fn place_item(
    img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    pos: u32,
    image_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let icon_size = GRID_SIZE * RATIO;

    let item = ImageReader::open(image_path)?.decode()?.resize_exact(
        icon_size,
        icon_size,
        image::imageops::FilterType::Nearest,
    );

    let (x, y) = grid_position(pos);
    for (ix, iy, src) in item.pixels() {
        let x = x + ix;
        let y = y + iy;
        let mut px = img.get_pixel(x, y).to_rgba();
        px.blend(&src);
        img.put_pixel(x, y, px.to_rgb());
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let base = ImageReader::open("crafting_table.png")?.decode()?;
    let base = base
        .resize_exact(
            base.width() * RATIO,
            base.height() * RATIO,
            image::imageops::FilterType::Nearest,
        )
        .to_rgb8();

    let file = File::create("out.gif")?;
    let mut file = BufWriter::new(file);
    let mut gif_encoder = Encoder::new(
        &mut file,
        base.width().try_into()?,
        base.height().try_into()?,
        &[],
    )?;

    gif_encoder.set_repeat(gif::Repeat::Infinite)?;

    let width = base.width();
    let height = base.height();

    macro_rules! foo {
        ($ty: literal) => {{
            let mut img = base.clone();

            move || -> anyhow::Result<_> {
                place_item(&mut img, 0, concat!("textures/", $ty, "_planks.png"))?;
                place_item(&mut img, 1, concat!("textures/", $ty, "_planks.png"))?;
                place_item(&mut img, 2, concat!("textures/", $ty, "_planks.png"))?;
                place_item(&mut img, 3, concat!("textures/", $ty, "_planks.png"))?;
                place_item(&mut img, 4, "textures/chest.png")?;
                place_item(&mut img, 5, concat!("textures/", $ty, "_planks.png"))?;
                place_item(&mut img, 6, concat!("textures/", $ty, "_planks.png"))?;
                place_item(&mut img, 7, concat!("textures/", $ty, "_planks.png"))?;
                place_item(&mut img, 8, concat!("textures/", $ty, "_planks.png"))?;

                place_item(&mut img, 9, "textures/barrel.png")?;

                let mut frame = Frame::from_rgb(width as _, height as _, img.as_raw());
                frame.delay = 100;
                anyhow::Result::Ok(frame)
            }
        }};
    }

    let mut threads = Vec::new();
    threads.push(std::thread::spawn(foo!("oak")));
    threads.push(std::thread::spawn(foo!("spruce")));
    threads.push(std::thread::spawn(foo!("birch")));
    threads.push(std::thread::spawn(foo!("jungle")));
    threads.push(std::thread::spawn(foo!("acacia")));
    threads.push(std::thread::spawn(foo!("dark_oak")));
    threads.push(std::thread::spawn(foo!("mangrove")));
    threads.push(std::thread::spawn(foo!("cherry")));
    threads.push(std::thread::spawn(foo!("bamboo")));
    threads.push(std::thread::spawn(foo!("crimson")));
    threads.push(std::thread::spawn(foo!("warped")));

    threads
        .into_iter()
        .try_for_each(|frame| -> anyhow::Result<_> {
            gif_encoder.write_frame(&frame.join().unwrap().unwrap())?;
            Ok(())
        })?;

    Ok(())
}
