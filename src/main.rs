extern crate doom_iow;
use doom_iow::*;

fn wall_x_texcoord(x: f32, y: f32, tex_walls: &Texture) -> i32 {
    let hitx = x - f32::floor(x + 0.5);
    let hity = y - f32::floor(y + 0.5);

    let x_texcoord: i32 = if f32::abs(hity) > f32::abs(hitx) {
        (hity * tex_walls.size as f32) as i32
    } else {
        (hitx * tex_walls.size as f32) as i32
    };
    let x_texcoord = if x_texcoord < 0 {
        x_texcoord + tex_walls.size as i32
    } else {
        x_texcoord
    };

    assert!(x_texcoord >= 0 && x_texcoord < tex_walls.size as i32);
    x_texcoord
}

fn render(
    fb: &mut Framebuffer,
    map: &Map,
    player: &Player,
    tex_walls: &Texture,
) -> Result<(), FrameError> {
    fb.clear(utils::pack_color_rgb(25, 25, 25));
    let rect_w = fb.w / (map.w * 2); //size of one map cell on the screen
    let rect_h = fb.h / map.h;
    for j in 0..map.h {
        for i in 0..map.w {
            if map.is_empty(i, j) {
                continue; //skip empty spaces
            }
            let rect_x = i * rect_w;
            let rect_y = j * rect_h;
            let texid = map.get(i, j).expect("i, j not in map range");
            fb.draw_rectangle(
                rect_x,
                rect_y,
                rect_w,
                rect_h,
                tex_walls.get(0, 0, texid).expect("no texture at texid"),
            )?;
        }
    }

    for i in 0..fb.w / 2 {
        //cast field of vision AND 3D view
        let angle: f32 = player.a - player.fov / 2. + player.fov * i as f32 / (fb.w / 2) as f32;
        for t in 0..2000 {
            //since Rust doesn't allow step by float, remap so step==1
            let t = t as f32 / 100.; //then transform back to original range

            let x = player.x + t * angle.cos();
            let y = player.y + t * angle.sin();

            //draw the visibility cone on the map
            fb.set_pixel(
                (x * rect_w as f32) as u32,
                (y * rect_h as f32) as u32,
                utils::pack_color_rgb(160, 160, 160),
            )
            .expect("Could not set pixel");

            if map.is_empty(x as u32, y as u32) {
                continue;
            }

            //if this map tile isn't empty, we've hit a wall
            //hit a wall
            let texid = map
                .get(x as u32, y as u32)
                .expect("Cannot index this map tile");
            assert!(texid < tex_walls.count);
            let column_height = (fb.h as f32 / (t * f32::cos(angle - player.a))) as u32;
            let x_texcoord = wall_x_texcoord(x, y, tex_walls);

            let column = tex_walls
                .get_scaled_column(texid, x_texcoord as u32, column_height)
                .expect("Cannot retrieve scaled column");

            let pix_x = i + fb.w / 2;
            for j in 0..column_height {
                let pix_y = j + fb.h / 2 - column_height / 2;
                if pix_y < fb.h {
                    fb.set_pixel(pix_x, pix_y, column[j as usize])
                        .expect("Could not set pixel");
                }
            }
            break;
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut fb = Framebuffer::new(1024, 512);

    let mut player = Player {
        x: 3.456,
        y: 2.345,
        a: 1.523,
        fov: std::f32::consts::PI / 3.,
    };

    let map = match Map::init(16, 16) {
        Ok(m) => m,
        Err(_) => {
            println!("bad parameters given to map");
            panic!("Could not open map");
        }
    };

    let tex_walls = Texture::new("./walltex.png").expect("Could not open texture in main");

    for frame in 0..360 {
        let output_path = "./out/";
        let ss = format!("{}{:05}.ppm", output_path, frame);
        player.a += 2. * std::f32::consts::PI / 360.;
        render(&mut fb, &map, &player, &tex_walls).expect("Could not render image");
        utils::drop_ppm_image(ss.as_str(), &fb.img, fb.w as usize, fb.h as usize)
            .expect("Could not drop image");
        println!("Rendered frame {}", frame);
    }
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

        utils::unpack_color(packed, &mut r, &mut g, &mut b, &mut a);

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

        utils::unpack_color(color, &mut rc, &mut gc, &mut bc, &mut ac);

        assert_eq!(vec![r, g, b, a], vec![rc, gc, bc, ac]);
    }
}
