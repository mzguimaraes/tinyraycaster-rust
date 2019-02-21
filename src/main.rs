

use doom_iow;
use doom_iow::framebuffer;
use doom_iow::utils;


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

fn load_texture(
    filename: &str,
    texture: &mut Vec<u32>,
    tex_size: &mut usize,
    tex_cnt: &mut usize,
) -> Result<Box<Vec<u32>>, image::ImageError> {
    let pixmap = image::open(filename)?
        .as_rgba8()
        .expect("Cannot open texture")
        .to_owned();

    let w = pixmap.width() as usize;
    let h = pixmap.height() as usize;

    *tex_cnt = (w / h) as usize;
    *tex_size = w as usize / *tex_cnt;
    if w != h * *tex_cnt {
        drop(texture);
        return Err(image::ImageError::FormatError(String::from(
            "Error: the texture file must contain N square textures packed horizontally",
        )));
    }

    let mut texture = vec![0; (w * h) as usize];
    let pixmap = pixmap.to_vec();
    for j in 0..h {
        for i in 0..w {
            let r = pixmap[(i + j * w) * 4 + 0];
            let g = pixmap[(i + j * w) * 4 + 1];
            let b = pixmap[(i + j * w) * 4 + 2];
            let a = pixmap[(i + j * w) * 4 + 3];
            texture[i + j * w] = utils::pack_color_rgba(r, g, b, a);
        }
    }
    Ok(Box::new(texture))
}

fn texture_column(
    img: &Vec<u32>,
    texsize: usize,
    ntextures: usize,
    texid: usize,
    texcoord: usize,
    column_height: usize,
) -> Vec<u32> {
    let img_w = texsize * ntextures;
    let img_h = texsize;
    assert!(img.len() == img_w * img_h && texcoord < texsize && texid < ntextures);
    let mut column = vec![0; column_height];

    for y in 0..column_height {
        let pix_x = texid * texsize + texcoord;
        let pix_y = (y * texsize) / column_height;
        column[y] = img[pix_x + pix_y * img_w];
    }

    column
}

fn main() -> std::io::Result<()> {
    let win_w: usize = 1024;
    let win_h: usize = 512;
    let mut framebuffer: Vec<u32> = vec![utils::pack_color_rgb(255, 255, 255); win_w * win_h];

    let map_w: usize = 16;
    let map_h: usize = 16;
    let map: Vec<char> = "0000222222220000\
                          1              0\
                          1      11111   0\
                          1     0        0\
                          0     0  1110000\
                          0     3        0\
                          0   10000      0\
                          0   3   11100  0\
                          5   4   0      0\
                          5   4   1  00000\
                          0       1      0\
                          2       1      0\
                          0       0      0\
                          0 0000000      0\
                          0              0\
                          0002222222200000"
        .chars()
        //.split("")
        .collect(); // our game map
                    //strip out null chars
                    // println!("{:?}", map);
                    //map.remove(map_w * map_h - 1);
                    //map.remove(0);

    assert_eq!(map.len(), map_w * map_h);

    let player_x = 3.456;
    let player_y = 2.345;
    let mut player_a: f64 = 1.523; //player view direction
    let fov = std::f64::consts::PI / 3.;

    let mut walltex: Box<Vec<u32>> = Box::new(Vec::new());
    let mut walltex_size: usize = 0;
    let mut walltex_cnt: usize = 0;
    walltex = match load_texture(
        "./walltex.png",
        &mut walltex,
        &mut walltex_size,
        &mut walltex_cnt,
    ) {
        Ok(tex) => tex,
        Err(e) => {
            println!("error loading texture: {}", e);
            Box::new(vec![utils::pack_color_rgb(100, 100, 100), 64 * 64 * 6]) // default texture
        }
    };

    //make immutable
    let walltex = walltex;
    let walltex_size = walltex_size;
    let walltex_cnt = walltex_cnt;

    let rect_w = win_w / (2 * map_w);
    let rect_h = win_h / map_h;

    // for frame in 0..360 {
    //     let output_path = "./out/";
    //     let ss = format!("{}{:05}", output_path, frame);
    //     player_a += 2. * std::f64::consts::PI / 360.;

    //     framebuffer = vec![pack_color_rgb(255, 255, 255); win_w * win_h]; //clear screen

    //draw the map
    for j in 0..map_h {
        for i in 0..map_w {
            //skip empty spaces
            if map[i + j * map_w] == ' ' {
                continue;
            }

            let rect_x = i * rect_w;
            let rect_y = j * rect_h;
            let texid: usize = map[i + j * map_w].to_digit(10).unwrap() as usize;
            assert!(texid < walltex_cnt);
            draw_rectangle(
                &mut framebuffer,
                win_w,
                win_h,
                rect_x,
                rect_y,
                rect_w,
                rect_h,
                walltex[texid * walltex_size],
            );
        }
    }

    for i in 0..win_w / 2 {
        //cast field of vision AND 3D view
        let angle: f64 = player_a - fov / 2. + fov * i as f64 / (win_w / 2) as f64;
        for t in 0..2000 {
            //since Rust doesn't allow step by float, remap so step==1
            let t = t as f64 / 100.; //then transform back to original range

            let cx = player_x + t * angle.cos();
            let cy = player_y + t * angle.sin();

            let mut pix_x = (cx * rect_w as f64) as i32;
            let mut pix_y = (cy * rect_h as f64) as i32;
            //draw the visibility cone on the map
            framebuffer[pix_x as usize + pix_y as usize * win_w] = utils::pack_color_rgb(160, 160, 160);

            if map[cx as usize + cy as usize * map_w] != ' ' {
                //hit a wall
                let texid: usize =
                    map[cx as usize + cy as usize * map_w].to_digit(10).unwrap() as usize;
                assert!(texid < walltex_cnt);
                let column_height = (win_h as f64 / (t * f64::cos(angle - player_a))) as usize;
                // draw_rectangle(
                //     &mut framebuffer,
                //     win_w,
                //     win_h,
                //     win_w / 2 + i,
                //     win_h / 2 - column_height / 2,
                //     1,
                //     column_height,
                //     walltex[texid * walltex_size],
                // );

                let hitx = cx - f64::floor(cx + 0.5);
                let hity = cy - f64::floor(cy + 0.5);
                let mut x_texcoord: i64 = (hitx * walltex_size as f64) as i64;
                if f64::abs(hity) > f64::abs(hitx) {
                    x_texcoord = (hity * walltex_size as f64) as i64;
                }

                if x_texcoord < 0 {
                    x_texcoord += walltex_size as i64;
                }
                assert!(x_texcoord >= 0 && x_texcoord < walltex_size as i64);

                let column = texture_column(
                    &walltex,
                    walltex_size,
                    walltex_cnt,
                    texid,
                    x_texcoord as usize,
                    column_height,
                );
                pix_x = win_w as i32 / 2 + i as i32;
                for j in 0..column_height {
                    pix_y = j as i32 + win_h as i32 / 2 - column_height as i32 / 2;
                    if pix_y < 0 || pix_y >= win_h as i32 {
                        continue;
                    }

                    framebuffer[pix_x as usize + pix_y as usize * win_w] = column[j];
                }

                break;
            }
        }
    }

    // draw the 4th texture on the screen (test)
    // let texid: usize = 4;
    // for i in 0..walltex_size {
    //     for j in 0..walltex_size {
    //         framebuffer[i + j * win_w] =
    //             walltex[i + texid * walltex_size + j * walltex_size * walltex_cnt];
    //     }
    // }

    utils::drop_ppm_image("./out.ppm", &framebuffer, win_w, win_h)?;

    // drop_ppm_image(ss.as_str(), &framebuffer, win_w, win_h)?;
    // }
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
        let packed = utils::pack_color_rgba(r, g, b, a);
        assert_eq!(packed, 0b00010000000010000000010000000010);
    }

    #[test]
    fn unpacks_ints() {
        let packed = 0b00010000000010000000010000000010;
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let mut a = 0;

        utils::unpack_color(&packed, &mut r, &mut g, &mut b, &mut a);

        assert_eq!(vec![2, 4, 8, 16], vec![r, g, b, a]);
    }

    #[test]
    fn packs_ints_idempotently() {
        let r = 2;
        let g = 4;
        let b = 8;
        let a = 255;

        let color = utils::pack_color_rgba(r, g, b, a);

        let mut rc = 0;
        let mut gc = 0;
        let mut bc = 0;
        let mut ac = 0;

        utils::unpack_color(&color, &mut rc, &mut gc, &mut bc, &mut ac);

        assert_eq!(vec![r, g, b, a], vec![rc, gc, bc, ac]);
    }
}
