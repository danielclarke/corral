use std::error::Error;
use std::fs;
use std::io::Write;

use crate::config::Config;
use crate::config::MetaDataFormat;
use crate::tree2d::{DataSize, Tree2d};
use image::{DynamicImage, ImageEncoder};

struct NamedDynamicImage {
    name: String,
    img: DynamicImage,
}

struct PackedImage {
    img: DynamicImage,
    meta_data: String,
}

impl PackedImage {
    fn write(
        &self,
        output_file: &str,
        output_file_format: MetaDataFormat,
    ) -> Result<(), Box<dyn Error>> {
        let buf = fs::File::create(output_file)?;
        let encoder = image::codecs::png::PngEncoder::new_with_quality(
            buf,
            image::codecs::png::CompressionType::Best,
            image::codecs::png::FilterType::Adaptive,
        );

        encoder.write_image(
            self.img.as_bytes(),
            self.img.width(),
            self.img.height(),
            self.img.color(),
        )?;

        let extension = match output_file_format {
            MetaDataFormat::Json => ".json",
            MetaDataFormat::Lua => ".lua",
        };

        let json_file = output_file.split('.').collect::<Vec<&str>>()[0].to_owned() + extension;
        let mut buf = fs::File::create(&json_file)?;
        match buf.write_all(self.meta_data.as_bytes()) {
            Ok(..) => Ok(()),
            Err(e) => Result::Err(Box::new(e)),
        }
    }
}

struct SpriteData {
    name: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl SpriteData {
    fn to_lua_string(&self) -> String {
        std::format!(
            "    {name} = {{
        x = {x},
        y = {y},
        width = {width},
        height = {height},
    }}",
            name = self.name.replace(' ', "_").to_uppercase(),
            x = self.x,
            y = self.y,
            width = self.width,
            height = self.height
        )
    }

    fn to_json_string(&self) -> String {
        std::format!(
            "{{\"height\":{height},\"name\":\"{name}\",\"width\":{width},\"x\":{x},\"y\":{y}}}",
            name = self.name.replace(' ', "_"),
            x = self.x,
            y = self.y,
            width = self.width,
            height = self.height
        )
    }
}

#[allow(dead_code)]
struct ImageCollection {
    named_images: Vec<NamedDynamicImage>,
    max_width: u32,
    max_height: u32,
    num_images: u32,
}

impl ImageCollection {
    fn new(mut named_images: Vec<NamedDynamicImage>) -> ImageCollection {
        let mut max_width = 0u32;
        let mut max_height = 0u32;
        for NamedDynamicImage { name: _, img } in &named_images {
            max_width = max_width.max(img.width());
            max_height = max_height.max(img.height());
        }
        let num_images = named_images.len() as u32;

        named_images.sort_by(|a, b| {
            (b.img.width() * b.img.height()).cmp(&(a.img.width() * a.img.height()))
        });

        ImageCollection {
            named_images,
            max_width,
            max_height,
            num_images,
        }
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let img_collection = load_all(&config.input_dir)?;
    let packed_img = pack(
        &config.output_file,
        config.output_file_format,
        config.padding,
        img_collection,
    )?;
    packed_img.write(&config.output_file, config.output_file_format)?;
    Ok(())
}

fn load_all(input_dir: &str) -> Result<ImageCollection, Box<dyn Error>> {
    let mut images = Vec::new();

    let paths = fs::read_dir(input_dir)?;

    for path in paths {
        let path = path?.path();
        if let (Some(path_str), Some(fname), Some(ext)) =
            (path.to_str(), path.file_prefix(), path.extension())
        {
            if ext != "png" {
                println!("Not png - skipping {path_str}");
                continue;
            }
            match image::io::Reader::open(path_str) {
                Ok(reader) => match reader.decode() {
                    Ok(img) => {
                        images.push(NamedDynamicImage {
                            name: fname.to_string_lossy().to_string(),
                            img,
                        });
                    }
                    Err(err) => {
                        eprintln!("Error decoding {path_str}");
                        return Err(Box::new(err));
                    }
                },
                Err(err) => {
                    eprintln!("Error opening {path_str}");
                    return Err(Box::new(err));
                }
            }
        }
    }

    Ok(ImageCollection::new(images))
}

fn pack(
    output_file_name: &str,
    output_file_format: MetaDataFormat,
    padding: u8,
    img_collection: ImageCollection,
) -> Result<PackedImage, Box<dyn Error>> {
    let mut data = vec![];
    for named_img in img_collection.named_images.iter() {
        data.push((
            DataSize {
                width: named_img.img.width() + padding as u32,
                height: named_img.img.height() + padding as u32,
            },
            named_img,
        ));
    }
    let mut tree = Tree2d::<&NamedDynamicImage>::new();
    tree.insert_all(data)?;
    let flattened = tree.flatten();
    let bb = tree.get_total_bounding_box();
    let mut img_packed =
        image::RgbaImage::new(bb.width + padding as u32, bb.height + padding as u32);
    let mut sprite_data = vec![];

    for (named_img, bb) in flattened {
        let x = bb.x as i64 + padding as i64;
        let y = bb.y as i64 + padding as i64;
        image::imageops::replace(&mut img_packed, &named_img.img, x, y);

        let sd = SpriteData {
            name: named_img.name.to_owned(),
            x: x as u32,
            y: y as u32,
            width: named_img.img.width(),
            height: named_img.img.height(),
        };
        sprite_data.push(sd);
    }

    sprite_data.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());

    let meta_data = match output_file_format {
        MetaDataFormat::Json => {
            let json_string: String = sprite_data
                .iter()
                .map(|sd| sd.to_json_string())
                .collect::<Vec<String>>()
                .join(",");
            "[".to_owned() + &json_string + "]\n"
        }
        MetaDataFormat::Lua => {
            let module_name = output_file_name.split('.').collect::<Vec<&str>>()[0].to_owned();
            let lua_string: String = sprite_data
                .iter()
                .map(|sd| sd.to_lua_string())
                .collect::<Vec<String>>()
                .join(",\n");
            format!("local {fname} = {{\n", fname = module_name)
                + &lua_string
                + &format!("\n}}\n\nreturn {fname}\n", fname = module_name)
        }
    };

    Ok(PackedImage {
        img: DynamicImage::ImageRgba8(img_packed),
        meta_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rect(w: u32, h: u32) -> image::DynamicImage {
        let mut img = image::RgbaImage::new(w, h);
        for i in 0..w {
            for j in 0..h {
                img.put_pixel(i, j, image::Rgba([255, 0, 0, 255]));
            }
        }
        image::DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn pack_one() -> Result<(), Box<dyn Error>> {
        let (w, h) = (1, 1);
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

        let img_collection = ImageCollection::new(vec![NamedDynamicImage {
            name: "red_pixel".to_owned(),
            img: make_rect(w, h),
        }]);

        if let Some(img) = pack(
            &"out.png",
            MetaDataFormat::Json,
            padding as u8,
            img_collection,
        )?
        .img
        .as_rgba8()
        {
            let p: Vec<&image::Rgba<u8>> = img.pixels().collect();
            let q: Vec<&image::Rgba<u8>> = expected_output_img.pixels().collect();
            assert_eq!(q, p);
        }
        Ok(())
    }

    // #[test]
    // fn pack_many() -> Result<(), Box<dyn Error>> {
    //     let dims = vec![
    //         (128, 96),
    //         (96, 128),
    //         (64, 96),
    //         (96, 64),
    //         (64, 64),
    //         (96, 96),
    //         (256, 64),
    //         (32, 32),
    //         (32, 32),
    //         (32, 32),
    //         (32, 32),
    //         (42, 42),
    //         (42, 42),
    //         (42, 42),
    //         (42, 42),
    //         (42, 42),
    //         (42, 42),
    //         (128, 96),
    //         (96, 128),
    //         (64, 96),
    //         (96, 64),
    //         (64, 64),
    //         (96, 96),
    //         (256, 64),
    //         (32, 32),
    //         (32, 32),
    //         (32, 32),
    //         (32, 32),
    //         (42, 42),
    //         (42, 42),
    //         (42, 42),
    //         (42, 42),
    //         (42, 42),
    //         (42, 42),
    //         (128, 96),
    //         (96, 128),
    //         (64, 96),
    //         (96, 64),
    //         (64, 64),
    //         (96, 96),
    //         (256, 64),
    //         (32, 32),
    //         (32, 32),
    //         (32, 32),
    //         (32, 32),
    //         (42, 42),
    //         (42, 42),
    //         (42, 42),
    //         (42, 42),
    //         (42, 42),
    //         (42, 42),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (16, 16),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //         (8, 8),
    //     ];
    //     let mut imgs = vec![];
    //     for (i, (w, h)) in (dims).iter().enumerate() {
    //         imgs.push(NamedDynamicImage {
    //             name: i.to_string(),
    //             img: make_rect(*w, *h),
    //         })
    //     }
    //     let img_collection = ImageCollection::new(imgs);
    //     let img_packed = pack(2, img_collection)?;
    //     img_packed.write("many.png")?;
    //     Ok(())
    // }
}
