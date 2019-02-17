use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn pack_color_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    let b1 = r as u32;
    let b2 = g as u32;
    let b3 = b as u32;
    let b4 = a as u32;
    (b4 << 24) + (b3 << 16) + (b2 << 8) + b1
}

//rust does not have operator overloading
fn pack_color_rgb(r: u8, g: u8, b: u8) -> u32 {
    pack_color_rgba(r, g, b, 255)
}

fn unpack_color(color: &u32, r: &mut u8, g: &mut u8, b: &mut u8, a: &mut u8) {
    *r = (color & 255) as u8; //keep last 8 bits
    *g = (color.rotate_right(8) & 255) as u8;
    *b = (color.rotate_right(16) & 255) as u8;
    *a = (color.rotate_right(24) & 255) as u8;
}

fn drop_ppm_image(filename: &str, image: &Vec<u32>, w: usize, h: usize) -> std::io::Result<()> {
    assert_eq!(image.len(), w * h);
    let output = File::open(filename);
    let output = match output {
        Ok(file) => file,
        Err(_) => File::create(filename).expect("Cannot open or create file"),
    };
    let mut output = BufWriter::new(output);
    let header = format!("P6\n{} {}\n255\n", w, h);

    output
        .write(header.as_bytes())
        .expect("cannot write header");

    for i in 0..w * h {
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let mut a = 0;
        unpack_color(&image[i], &mut r, &mut g, &mut b, &mut a);

        output.write(&[r, g, b])?;
    }
    //output closes at end of scope
    Ok(())
}

fn draw_rectangle(
    img: &mut Vec<u32>,
    img_w: usize,
    img_h: usize,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    color: u32,
) {
    for i in 0..w {
        for j in 0..h {
            let cx = x + i;
            let cy = y + j;

            if cx >= img_w || cy >= img_h {
                //don't write past size of image
                continue;
            }

            img[cx + cy * img_w] = color;
        }
    }
}

fn main() -> std::io::Result<()> {
    let filename = "./out.ppm";
    if Path::new(filename).exists() {
        fs::remove_file(filename)?;
    }

    let win_w: usize = 1024;
    let win_h: usize = 512;
    let mut framebuffer: Vec<u32> = vec![pack_color_rgb(60, 60, 60); win_w * win_h];

    let map_w: usize = 16;
    let map_h: usize = 16;
    let mut map: Vec<&str> = "0000222222220000\
                              1              0\
                              1      11111   0\
                              1     0        0\
                              0     0  1110000\
                              0     3        0\
                              0   10000      0\
                              0   0   11100  0\
                              0   0   0      0\
                              0   0   1  00000\
                              0       1      0\
                              2       1      0\
                              0       0      0\
                              0 0000000      0\
                              0              0\
                              0002222222200000"
        .split("")
        .collect(); // our game map
                    //strip out null chars
    map.remove(map_w * map_h - 1);
    map.remove(0);

    assert_eq!(map.len(), map_w * map_h);

    let player_x = 3.456;
    let player_y = 2.345;
    let player_a: f64 = 1.523; //player view direction
    let fov = std::f64::consts::PI / 3.;

    let rect_w = win_w / (2 * map_w);
    let rect_h = win_h / map_h;
    for j in 0..map_h {
        //draw the map
        for i in 0..map_w {
            if map[i + j * map_w] == " " {
                continue;
            } //skip empty spaces
            let rect_x = i * rect_w;
            let rect_y = j * rect_h;
            draw_rectangle(
                &mut framebuffer,
                win_w,
                win_h,
                rect_x,
                rect_y,
                rect_w,
                rect_h,
                pack_color_rgb(0, 255, 255),
            );
        }
    }

    for i in 0..win_w / 2 {
        //cast field of vision AND 3D view
        let angle: f64 = player_a - fov / 2. + fov * i as f64 / (win_w / 2) as f64;
        for t in 0..400 {
            //since Rust doesn't allow step by float, remap so step==1
            let t = t as f64 / 20.; //then transform back to original range

            let cx = player_x + t * angle.cos();
            let cy = player_y + t * angle.sin();
            angle.sin();

            let pix_x = (cx * rect_w as f64) as usize;
            let pix_y = (cy * rect_h as f64) as usize;
            //draw the visibility cone on the map
            framebuffer[pix_x + pix_y * win_w] = pack_color_rgb(255, 255, 255);

            if map[cx as usize + cy as usize * map_w] != " " {
                let column_height = (win_h as f64 / t) as usize;
                draw_rectangle(
                    &mut framebuffer,
                    win_w,
                    win_h,
                    win_w / 2 + i,
                    win_h / 2 - column_height / 2,
                    1,
                    column_height,
                    pack_color_rgb(0, 255, 255),
                );
                break;
            }
        }
    }

    drop_ppm_image(filename, &framebuffer, win_w, win_h)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_ints() {
        let r = 2;
        let g = 4;
        let b = 8;
        let a = 16;
        let packed = pack_color_rgba(r, g, b, a);
        assert_eq!(packed, 0b00010000000010000000010000000010);
    }

    #[test]
    fn unpacks_ints() {
        let packed = 0b00010000000010000000010000000010;
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let mut a = 0;

        unpack_color(&packed, &mut r, &mut g, &mut b, &mut a);

        assert_eq!(vec![2, 4, 8, 16], vec![r, g, b, a]);
    }

    #[test]
    fn packs_ints_idempotently() {
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
