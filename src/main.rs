use image::{open, RgbImage, Rgb, GrayImage, Luma};

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
    elevation: &GrayImage,
    cell_size: f64,         // Distance between pixels (e.g., meters)
    azimuth_deg: f64,       // Direction of light source (e.g., 315 = NW)
    altitude_deg: f64,      // Altitude of sun above horizon (e.g., 45)
) -> RgbImage {
    let (width, height) = elevation.dimensions();
    let mut shaded = RgbImage::new(width, height);

    let azimuth_rad = (360.0 - azimuth_deg + 90.0).to_radians();
    let altitude_rad = altitude_deg.to_radians();

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let z = |dx: i32, dy: i32| {
                let px = (x as i32 + dx) as u32;
                let py = (y as i32 + dy) as u32;
                elevation.get_pixel(px, py)[0] as f64
            };

            let dz_dx = ((z(1, -1) + 2.0 * z(1, 0) + z(1, 1)) - (z(-1, -1) + 2.0 * z(-1, 0) + z(-1, 1))) / (8.0 * cell_size);
            let dz_dy = ((z(-1, 1) + 2.0 * z(0, 1) + z(1, 1)) - (z(-1, -1) + 2.0 * z(0, -1) + z(1, -1))) / (8.0 * cell_size);

            let slope_rad = (dz_dx * dz_dx + dz_dy * dz_dy).sqrt().atan();
            let aspect_rad = dz_dy.atan2(-dz_dx);

            let shaded_val = 255.0 * (
                altitude_rad.sin() * slope_rad.sin() +
                altitude_rad.cos() * slope_rad.cos() * (azimuth_rad - aspect_rad).cos()
            );

            let intensity = shaded_val.max(0.0).min(255.0) as u8;
            shaded.put_pixel(x, y, Rgb([intensity, intensity, intensity]));
        }
    }

    shaded
}