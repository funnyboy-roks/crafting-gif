#![allow(clippy::identity_op)]

mod cli;
mod config;

use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::Context;
use clap::Parser;
use gif::{Encoder, Frame};
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Pixel, Rgb};

use crate::config::Config;

const CRAFTING_TABLE_PNG: &[u8] = include_bytes!("../assets/crafting_table.png");
const CRAFTING_TABLE_DARK_PNG: &[u8] = include_bytes!("../assets/crafting_table_dark.png");

/// Size of a slot in `crafting_table.png`
const GRID_SIZE: u32 = 16;

/// The amount that `crafting_table.png` should be upscaled
const RATIO: u32 = 5;

const fn grid_position(grid_position: u32) -> (u32, u32) {
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

struct CountingWriter<W> {
    inner: W,
    count: usize,
}

impl<W> CountingWriter<W> {
    pub fn new(w: W) -> Self {
        Self { inner: w, count: 0 }
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl<W> Write for CountingWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf).inspect(|n| self.count += n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.inner.write_vectored(bufs).inspect(|n| self.count += n)
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.inner.write_all(buf)?;
        self.count += buf.len();
        Ok(())
    }

    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.inner.write_fmt(args)
    }
}

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    let config = fs::read_to_string(cli.recipe)?;
    let config: Config = toml::from_str(&config)?;

    let base = ImageReader::with_format(
        std::io::Cursor::new(if cli.dark {
            CRAFTING_TABLE_DARK_PNG
        } else {
            CRAFTING_TABLE_PNG
        }),
        image::ImageFormat::Png,
    )
    .decode()?;

    let base = base
        .resize_exact(
            base.width() * RATIO,
            base.height() * RATIO,
            image::imageops::FilterType::Nearest,
        )
        .to_rgb8();

    let mut file = CountingWriter::new(BufWriter::new(File::create(&cli.out)?));
    let mut gif_encoder = Encoder::new(
        &mut file,
        base.width().try_into()?,
        base.height().try_into()?,
        &[],
    )?;

    gif_encoder.set_repeat(gif::Repeat::Infinite)?;

    let width = base.width();
    let height = base.height();

    let mut threads = Vec::new();
    for i in 0..config.frames.get() {
        let mut img = base.clone();
        let (grid, result) = config.recipe(i as _)?;

        threads.push(std::thread::spawn(move || -> anyhow::Result<_> {
            for (i, x) in grid.into_iter().enumerate() {
                if let Some(x) = x {
                    place_item(&mut img, i as _, &x)
                        .with_context(|| format!("Placing '{}' at slot {}", x.display(), i))?;
                }
            }

            place_item(&mut img, 9, &result)
                .with_context(|| format!("placing result: '{}'", result.display()))?;

            let mut frame = Frame::from_rgb_speed(width as _, height as _, img.as_raw(), 5);
            frame.delay = config.frame_duration;
            anyhow::Result::Ok(frame)
        }));
    }

    threads
        .into_iter()
        .try_for_each(|frame| -> anyhow::Result<_> {
            let f = frame.join().unwrap()?;
            gif_encoder.write_frame(&f)?;
            Ok(())
        })?;

    eprintln!(
        "Wrote {} bytes to {}.",
        gif_encoder.into_inner()?.count(),
        cli.out.display()
    );

    Ok(())
}
