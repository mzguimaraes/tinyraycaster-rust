use std::fs::File;
use std::io::{BufWriter, Write};

fn pack_color_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    //((a << 24) + (b << 16) + (g << 8) + r) as u32
    let b1 = r as u32;
    let b2 = g as u32;
    let b3 = b as u32;
    let b4 = a as u32;
    (b4 << 24) + (b3 << 16) + (b2 << 8) + b1
}

fn pack_color_rgb(r: u8, g: u8, b: u8) -> u32 {
    pack_color_rgba(r, g, b, 255)
}

fn unpack_color(color: &u32, r: &mut u8, g: &mut u8, b: &mut u8, a: &mut u8) {
    *r = (color & 255) as u8; //keep last 8 bits
    *g = (color.rotate_right(8) & 255) as u8;
    *b = (color.rotate_right(16) & 255) as u8;
    *a = (color.rotate_right(24) & 255) as u8;
}

fn drop_ppm_image(filename: &str, image: &Vec<u32>, w: usize, h: usize) {
    assert_eq!(image.len(), w * h);
    let output = File::open(filename);
    let output = match output {
        Ok(file) => file,
        Err(_) => File::create(filename).expect("Cannot open or create file"),
    };
    let mut output = BufWriter::new(output);
    let header = format!("P6\n{} {}\n255\n", w, h);
    // fs::write(filename, header).expect("Unable to write to file");
    output.write(header.as_bytes()).unwrap();

    for i in 0..h * w {
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let mut a = 0;
        unpack_color(&image[i as usize], &mut r, &mut g, &mut b, &mut a);

        output
            .write(format!("{} {} {}", r, g, b).as_bytes())
            .unwrap();
    }

    //output closes at end of scope
}

fn main() {
    let win_w: usize = 512;
    let win_h: usize = 512;
    let mut framebuffer: Vec<u32> = vec![255; win_w * win_h];

    for j in 0..win_h {
        for i in 0..win_w {
            //let r: u8 = (255.0 * (j as f32) / (win_h as f32)) as u8;
            //let g: u8 = (255.0 * (i as f32) / (win_w as f32)) as u8;
            let r: u8 = 255;
            let g: u8 = 255;
            let b: u8 = 0;
            framebuffer[i + j * win_w] = pack_color_rgb(r, g, b);
        }
    }

    drop_ppm_image("./out.ppm", &framebuffer, win_w, win_h);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_ints() {
        let r = 2;
        let g = 4;
        let b = 8;
        let a = 255;

        let color = pack_color_rgba(r, g, b, a);

        let mut rc = 0;
        let mut gc = 0;
        let mut bc = 0;
        let mut ac = 0;

        unpack_color(&color, &mut rc, &mut gc, &mut bc, &mut ac);

        assert_eq!(vec![r, g, b, a], vec![rc, gc, bc, ac]);
    }
}
