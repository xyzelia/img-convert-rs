use std::{fs::File, io, path::PathBuf};
use image::{ EncodableLayout};
use std::io::Write;
use std::path::Path;
use image::{DynamicImage, ImageReader};
use webp::{Encoder, WebPMemory};
use std::fs;

pub fn image_to_webp(file_path:&PathBuf, export_path:&PathBuf, quality:f32, lossless:bool) ->Option<String> {
    let image = ImageReader::open(file_path);
    let mut image: DynamicImage  =match image {
        Ok(img) => img.with_guessed_format().unwrap().decode().unwrap(), //ImageReader::with_guessed_format() function guesses if image needs to be opened in JPEG or PNG format.
        Err(e) => {
            println!("Error: {}", e);
            return None;
        }
    };

    // 转换成 DynamicImage::ImageRgba8
    // 检查格式，如果是 JPG / JPEG，就转成 RGBA8 ，避免 Encoder 报 "Unimplemented"
    match ImageReader::open(file_path).and_then(|r| r.with_guessed_format()) {
        Ok(r) => {
            match r.format() {
                Some(fmt) if fmt == image::ImageFormat::Jpeg => {
                    image = DynamicImage::ImageRgba8(image.to_rgba8());
                }
                _ => {} // 其它格式不处理
            }
        }
        Err(e) => {
            println!("Error guessing format: {}", e);
        }
    }


    let encoder: Encoder = Encoder::from_image(&image).unwrap();
    let encoded_webp: WebPMemory = if lossless {
        encoder.encode_lossless()
    } else {
        encoder.encode(quality)  // Use lossy encoding
    };

    // let encoded_webp: WebPMemory = encoder.encode(quality);

    // Put webp-image in a separate webp-folder in the location of the original image.
    let path: &Path = Path::new(file_path);
    let parent_directory: &Path = path.parent().unwrap();
    let webp_folder_path = format!("{}/webp", parent_directory.to_str().unwrap());
    match std::fs::create_dir_all(webp_folder_path.to_string()) {
        Ok(_) => {}
        Err(e) => {
            println!("Error {}", e);
            return None;
        }
    }

    // Make full output path for webp-image.
    let webp_image_path = export_path;

    // Make File-stream for WebP-result and write bytes into it, and save to path "output.webp".
    let mut webp_image = File::create(webp_image_path).unwrap();
    match webp_image.write_all(encoded_webp.as_bytes()) {
        Err(e) => {
            println!("Error: {}", e);
            return None;
        }
        Ok(_) => Some(webp_image_path.display().to_string()),
    }
}