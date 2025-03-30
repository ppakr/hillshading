use image::{open, RgbImage, Rgb, GrayImage};
use std::f32::consts::PI;

fn main() {
    let img = open("output.png").expect("Failed to open image");

    // Convert grayscale to RGB
    let color_img = grayscale_to_colormap(&img.to_luma8());
    // Save the colorized image
    color_img.save("output_colorized.png").expect("Failed to save image");

    // Hill shading
    let dem = image::open("output.png").expect("Failed to open DEM image").to_luma8();
    let hillshade = apply_hillshade(&dem, 30.0, 315.0, 45.0); // 30m cell size, NW light, 45° sun
    hillshade.save("hillshade.png").expect("Failed to save hillshade image");

    // blend
    let blended = combine(&color_img, &hillshade);
    blended.save("blended.png").expect("Failed to save blended image");
}

fn deg2rad(deg : f32)-> f32 {
    deg * (PI / 180.0)
}

fn grayscale_to_colormap(gray: &GrayImage) -> RgbImage {
    let (width, height) = gray.dimensions();
    let mut rgb_img = RgbImage::new(width, height);

    for (x, y, pixel) in gray.enumerate_pixels() {
        let v = pixel[0] as f32 / 255.0;

        // Example gradient: dark green → yellow → white
        let r = (v * 255.0) as u8;
        let g = ((1.0 - (v - 0.5).abs()) * 255.0) as u8;
        let b = ((1.0 - v) * 128.0) as u8;

        rgb_img.put_pixel(x, y, Rgb([r, g, b]));
    }

    rgb_img
}

fn apply_hillshade(
    gray: &GrayImage,
    cell_size: f32,         // Distance between pixels 
    azimuth_deg: f32,       // Direction of light source 315)
    altitude_deg: f32,      // Altitude of sun above horizon (45)
) -> RgbImage {
    let (width, height) = gray.dimensions();
    let mut rgb_img = RgbImage::new(width, height);

    let az_rad = deg2rad(360.0 - azimuth_deg + 90.0) % (2.0 * PI); // convert to math azimuth
    let alt_rad = deg2rad(altitude_deg);

    let get = |x: i32, y: i32| -> f32 {
        let cx = x.clamp(0, width as i32 - 1) as u32;
        let cy = y.clamp(0, height as i32 - 1) as u32;
        gray.get_pixel(cx, cy)[0] as f32
    };

    for y in 0..height as i32 {
        for x in 0..width as i32 {
            // Get elevation from 3x3 neighborhood
            let z1 = get(x - 1, y - 1);
            let z2 = get(x,     y - 1);
            let z3 = get(x + 1, y - 1);
            let z4 = get(x - 1, y);
            let z5 = get(x + 1, y);
            let z6 = get(x - 1, y + 1);
            let z7 = get(x,     y + 1);
            let z8 = get(x + 1, y + 1);

            // Horn's method for dz/dx and dz/dy
            let dzdx = ((z3 + 2.0 * z5 + z8) - (z1 + 2.0 * z4 + z6)) / (8.0 * cell_size);
            let dzdy = ((z6 + 2.0 * z7 + z8) - (z1 + 2.0 * z2 + z3)) / (8.0 * cell_size);

            let slope = (dzdx * dzdx + dzdy * dzdy).sqrt().atan();
            let aspect = if dzdx != 0.0 {
                let mut aspect = (dzdy / -dzdx).atan();
                if dzdx > 0.0 {
                    aspect += PI;
                } else if dzdy < 0.0 {
                    aspect += 2.0 * PI;
                }
                aspect
            } else if dzdy > 0.0 {
                PI / 2.0
            } else {
                3.0 * PI / 2.0
            };

            let hs = (alt_rad.cos() * slope.cos()
                + alt_rad.sin() * slope.sin() * (az_rad - aspect).cos()).max(0.0);

            let shade = (255.0 * hs).round() as u8;

            // Optional: apply to grayscale colormap or RGB
            rgb_img.put_pixel(x as u32, y as u32, Rgb([shade, shade, shade]));
        }
    }

    rgb_img
}

fn combine(colormap:&RgbImage, hillshade:&RgbImage) -> RgbImage {
    let (width, height) = colormap.dimensions();
    let mut my_special_blend = RgbImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let color = colormap.get_pixel(x, y);
            let shade = hillshade.get_pixel(x, y)[0] as f32 / 255.0;

            let r = (color[0] as f32 * shade).clamp(0.0, 255.0) as u8;
            let g = (color[1] as f32 * shade).clamp(0.0, 255.0) as u8;
            let b = (color[2] as f32 * shade).clamp(0.0, 255.0) as u8;

            my_special_blend.put_pixel(x, y, Rgb([r, g, b]));
        }
    }

    my_special_blend
}
