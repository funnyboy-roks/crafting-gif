use std::path::Path;

use image::{GenericImageView, ImageBuffer, ImageReader, Pixel, Rgb};

const GRID_SIZE: u32 = 16;

const RATIO: u32 = 5;

fn grid_position(grid_position: u32) -> (u32, u32) {
    let icon_size = GRID_SIZE * RATIO;

    let (cx, cy) = if grid_position == 9 {
        // corner of output slot is (93, 17)
        // Output slot is 24x
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
    let item = ImageReader::open(image_path)?.decode()?;

    let icon_size = GRID_SIZE * RATIO;

    let item_rz = item.resize_exact(icon_size, icon_size, image::imageops::FilterType::Nearest);

    let (x, y) = grid_position(pos);
    for (ix, iy, src) in item_rz.pixels() {
        let x = x + ix;
        let y = y + iy;
        let mut px = img.get_pixel(x, y).to_rgba();
        px.blend(&src);
        img.put_pixel(x, y, px.to_rgb());
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let img = ImageReader::open("crafting_table.png")?.decode()?;
    let mut img = img
        .resize_exact(
            img.width() * RATIO,
            img.height() * RATIO,
            image::imageops::FilterType::Nearest,
        )
        .to_rgb8();

    place_item(&mut img, 0, "textures/oak_planks.png")?;
    place_item(&mut img, 1, "textures/oak_planks.png")?;
    place_item(&mut img, 2, "textures/oak_planks.png")?;
    place_item(&mut img, 3, "textures/oak_planks.png")?;
    place_item(&mut img, 4, "textures/chest.png")?;
    place_item(&mut img, 5, "textures/oak_planks.png")?;
    place_item(&mut img, 6, "textures/oak_planks.png")?;
    place_item(&mut img, 7, "textures/oak_planks.png")?;
    place_item(&mut img, 8, "textures/oak_planks.png")?;

    place_item(&mut img, 9, "textures/barrel.png")?;

    // let img: DynamicImage = img.into();
    // let img = img.resize(512, 800, image::imageops::FilterType::Nearest);

    img.save("out.png")?;

    Ok(())
}
