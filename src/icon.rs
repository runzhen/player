use image::{ImageBuffer, Rgba};

/// Generate a 64×64 RGBA icon: blue circle with a white music note.
/// Returns PNG bytes suitable for tauri::image::Image::from_bytes.
pub fn generate_icon_png() -> Vec<u8> {
    let size = 64u32;
    let center = size as f64 / 2.0;
    let radius = 28.0;

    let blue = Rgba([30u8, 120, 230, 255]);
    let white = Rgba([255u8, 255, 255, 255]);
    let transparent = Rgba([0u8, 0, 0, 0]);

    let mut img = ImageBuffer::from_pixel(size, size, transparent);

    // Draw blue circle
    for y in 0..size {
        for x in 0..size {
            let dx = x as f64 - center;
            let dy = y as f64 - center;
            if dx * dx + dy * dy <= radius * radius {
                img.put_pixel(x, y, blue);
            }
        }
    }

    // Draw white music note
    // Note head (ellipse) at bottom-left area
    let head_cx = 24.0f64;
    let head_cy = 44.0;
    let head_rx = 7.0;
    let head_ry = 5.0;

    for y in 0..size {
        for x in 0..size {
            let dx = (x as f64 - head_cx) / head_rx;
            let dy = (y as f64 - head_cy) / head_ry;
            if dx * dx + dy * dy <= 1.0 {
                img.put_pixel(x, y, white);
            }
        }
    }

    // Stem: vertical line from note head going up
    let stem_x_start = 30;
    let stem_x_end = 33;
    let stem_y_start = 18;
    let stem_y_end = 44;
    for y in stem_y_start..stem_y_end {
        for x in stem_x_start..stem_x_end {
            img.put_pixel(x, y, white);
        }
    }

    // Flag: small triangle/curve at top of stem
    for y in 18u32..30 {
        let flag_width = ((30.0 - y as f64) / 12.0 * 10.0) as u32;
        for x in 33..(33 + flag_width).min(size) {
            img.put_pixel(x, y, white);
        }
    }

    // Encode as PNG using the DynamicImage write_to method
    let dynamic = image::DynamicImage::ImageRgba8(img);
    let mut png_bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    dynamic
        .write_to(&mut cursor, image::ImageFormat::Png)
        .expect("Failed to encode icon PNG");
    png_bytes
}
