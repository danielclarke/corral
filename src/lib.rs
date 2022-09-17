use std::error::Error;
use std::fs;

use image::DynamicImage;

pub struct Config {
    padding: u8,
    input_dir: String,
    output_file: String,
}

impl Config {
    pub fn parse(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("Too few arguments, call like: `corral input_dir output_sheet.png`");
        }

        let input_dir = args[1].clone();
        let output_file = args[2].clone();

        Ok(Config {
            padding: 2u8,
            input_dir,
            output_file,
        })
    }
}

struct ImageCollection {
    images: Vec<DynamicImage>,
    max_width: u32,
    max_height: u32,
    num_images: u32,
}

impl ImageCollection {
    fn new(images: Vec<DynamicImage>) -> ImageCollection {
        let mut max_width = 0u32;
        let mut max_height = 0u32;
        for im in &images {
            max_width = max_width.max(im.width());
            max_height = max_height.max(im.height());
        }
        let num_images = (&images).len() as u32;
        ImageCollection {
            images,
            max_width,
            max_height,
            num_images,
        }
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let img_collection = load_all(&config.input_dir)?;
    let img_packed = pack(config.padding, &img_collection);
    img_packed.save(config.output_file)?;

    // for im in &(img_collection.images) {
    //     println!("w {}, h {}", im.width(), im.height());
    // }

    // println!("Reading images from {}", config.input_dir);
    // println!("Writing sprite sheet to {}", config.output_file);

    Ok(())
}

fn load_all(input_dir: &str) -> Result<ImageCollection, Box<dyn Error>> {
    let mut images = Vec::new();

    let paths = fs::read_dir(input_dir)?;

    for path in paths {
        let path = path?.path();
        if let Some(path_str) = path.to_str() {
            images.push(image::io::Reader::open(path_str)?.decode()?);
        }
    }

    Ok(ImageCollection::new(images))
}

fn pack(padding: u8, img_collection: &ImageCollection) -> DynamicImage {
    let h = img_collection.max_height + (padding * 2) as u32;
    let w = (img_collection.max_width + padding as u32) * img_collection.num_images
        + padding as u32;

    let mut packed_img = image::RgbaImage::new(w, h);

    for (i, img) in (img_collection.images).iter().enumerate() {
        image::imageops::replace(
            &mut packed_img,
            img,
            (i as i64) * (img_collection.max_width + padding as u32) as i64 + padding as i64,
            padding as i64,
        );
    }

    DynamicImage::ImageRgba8(packed_img)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_one() {
        let (w, h) = (1, 1);
        let mut img = image::RgbaImage::new(w, h);
        for i in 0..w {
            for j in 0..h {
                img.put_pixel(i, j, image::Rgba([255, 0, 0, 255]));
            }
        }

        let padding = 1;
        let mut expected_output_img = image::RgbaImage::new(w + padding * 2, h + padding * 2);
        for i in 0..w + padding * 2 {
            for j in 0..h + padding * 2 {
                let color = if i < padding || j < padding {
                    image::Rgba([0, 0, 0, 0])
                } else if i >= padding + w || j >= padding + h {
                    image::Rgba([0, 0, 0, 0])
                } else {
                    image::Rgba([255, 0, 0, 255])
                };
                expected_output_img.put_pixel(i, j, color);
            }
        }

        let img_collection = ImageCollection::new(vec![image::DynamicImage::ImageRgba8(img)]);

        if let Some(img) = pack(padding as u8, &img_collection).as_rgba8() {
            let p: Vec<&image::Rgba<u8>> = img.pixels().collect();
            let q: Vec<&image::Rgba<u8>> = expected_output_img.pixels().collect();
            assert_eq!(q, p);
        }
    }
}
