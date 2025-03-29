use image::{open, RgbImage, Rgb, GrayImage};

fn main() {
    let img = open("output.png").expect("Failed to open image");

    // Convert grayscale to RGB
    let color_img = grayscale_to_colormap(&img.to_luma8());
    // Save the colorized image
    color_img.save("output_colorized.png").expect("Failed to save image");
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