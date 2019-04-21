extern crate doom_iow;
use doom_iow::*;

use std::f32;
use std::process::Command;

// returns the RGBA color corresponding to UV(hitx, hity) on tex_walls
fn wall_x_texcoord(hitx: f32, hity: f32, tex_walls: &Texture) -> i32 {
    let x = hitx - f32::floor(hitx + 0.5);
    let y = hity - f32::floor(hity + 0.5);

    let x_texcoord: i32 = if f32::abs(y) > f32::abs(x) {
        (y * tex_walls.size as f32) as i32
    } else {
        (x * tex_walls.size as f32) as i32
    };
    let x_texcoord = if x_texcoord < 0 {
        x_texcoord + tex_walls.size as i32
    } else {
        x_texcoord
    };

    assert!(x_texcoord >= 0 && x_texcoord < tex_walls.size as i32);
    x_texcoord
}

fn draw_sprite(
    sprite: &Sprite,
    depth_buffer: &Vec<f32>,
    fb: &mut Framebuffer,
    player: &Player,
    tex_sprites: &Texture,
) -> Result<(), FrameError> {
    //absolute direction from player to sprite (rads)
    let mut sprite_dir = f32::atan2(sprite.y - player.y, sprite.x - player.x);
    //remap to range [-pi, pi]
    while sprite_dir >  f32::consts::PI { sprite_dir -= 2.0 * f32::consts::PI; }
    while sprite_dir < -f32::consts::PI { sprite_dir += 2.0 * f32::consts::PI; }

    //distance from player to sprite
    // let sprite_dist =
    //     f32::sqrt(f32::powi(player.x - sprite.x, 2) + f32::powi(player.y - sprite.y, 2));
    // let sprite_screen_size = f32::min(2000.0, fb.h as f32 / sprite_dist) as i32;
    let sprite_screen_size = f32::min(1000.0, fb.h as f32/sprite.player_dist) as i32;
    let screen_size = fb.w as i32 / 2;
    let h_offset: i32 = ((sprite_dir - player.get_a()) * (fb.w as f32/2.0)/(player.fov) + 
        (fb.w as f32/2.0)/2.0 - (sprite_screen_size as f32)/2.0) as i32;
    let v_offset: i32 = (fb.h as i32/2 - sprite_screen_size/2) as i32;

    // println!("h_offset = {} = ({} - {}) * {}/2/{} + {}/2/2 - {}/2", h_offset, sprite_dir, player.a, fb.w, player.fov, fb.w, sprite_screen_size);
    for i in 0..sprite_screen_size {
        if h_offset+i<0 || h_offset+i >= screen_size { continue; }
        if depth_buffer[(h_offset+i) as usize] < sprite.player_dist { continue; }
        for j in 0..sprite_screen_size {
            if v_offset+j<0 || v_offset+j >= fb.h as i32 { continue; }
            let color = tex_sprites.get(i as u32*tex_sprites.size/sprite_screen_size as u32, 
                j as u32*tex_sprites.size/sprite_screen_size as u32, sprite.tex_id)
                .unwrap();
            let ( _, _, _, a) = utils::unpack_color(color);
            if a > 128 {
                fb.set_pixel(fb.w/2 + (h_offset+i) as u32, (v_offset+j) as u32, color)?;
            }
        }
    }
    Ok(())
}

fn map_show_sprite(sprite: &Sprite, fb: &mut Framebuffer, map: &Map) -> Result<(), FrameError> {
    //(rect_w, rect_h) == size of one map tile
    let rect_w = (fb.w / (map.w * 2)) as f32;
    let rect_h = (fb.h / map.h) as f32;
    fb.draw_rectangle(
        (sprite.x * rect_w - 3.0) as u32,
        (sprite.y * rect_h - 3.0) as u32,
        6,
        6,
        utils::pack_color_rgb(255, 0, 0),
    )
}

fn render(
    fb: &mut Framebuffer,
    map: &Map,
    player: &Player,
    sprites: &mut Vec<Sprite>, // will change order of sprites according to distance from player
    tex_walls: &Texture,
    tex_monsters: &Texture,
) -> Result<(), FrameError> {
    fb.clear(utils::pack_color_rgb(249, 209, 152));
    let rect_w = fb.w / (map.w * 2); //size of one map cell on the screen
    let rect_h = fb.h / map.h;

    // draw overhead map
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

    let mut depth_buffer = vec![1e3; (fb.w/2) as usize];
    for i in 0..fb.w / 2 {
        //cast field of vision on map AND generate 3D view
        let angle: f32 = player.get_a() - player.fov / 2. + player.fov * i as f32 / (fb.w / 2) as f32;
        for t in 0..2000 {
            //since Rust doesn't allow step by float, remap so step==1
            let t = t as f32 / 100.; //then transform back to original range

            let x = player.x + t * angle.cos();
            let y = player.y + t * angle.sin();

            // draw the visibility cone on the map
            fb.set_pixel(
                (x * rect_w as f32) as u32,
                (y * rect_h as f32) as u32,
                utils::pack_color_rgb(160, 160, 160),
            )
            .expect("Could not set pixel");

            // if this map tile isn't empty, we've hit a wall
            if map.is_empty(x as u32, y as u32) {
                continue;
            }

            // hit a wall
            let texid = map
                .get(x as u32, y as u32)
                .expect("Cannot index this map tile");
            assert!(texid < tex_walls.count);

            let distance = t * f32::cos(angle - player.get_a());
            depth_buffer[i as usize] = distance;
            let column_height = (fb.h as f32 / distance) as u32;

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
    // update distances from sprites to player
    for sprite in sprites.iter_mut() {
        sprite.player_dist = f32::sqrt(f32::powi(player.x - sprite.x, 2) + f32::powi(player.y - sprite.y, 2));
    }
    // sort sprites in reverse order of distance to player
    sprites.sort_unstable_by(|lhs, rhs| rhs.player_dist.partial_cmp(&lhs.player_dist).unwrap());

    // render sprites on map
    for sprite in sprites.iter().take(sprites.len()) {
        map_show_sprite(sprite, fb, &map)?;
        draw_sprite(sprite, &depth_buffer, fb, &player, &tex_monsters)?;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    //clear the /out folder
    Command::new("rm")
        .arg("-rf")
        .arg("out/")
        .output()
        .expect("failed to clear out directory");

    //create new /out folder
    Command::new("mkdir")
        .arg("out")
        .output()
        .expect("failed to create directory");

    let mut fb = Framebuffer::new(1024, 512);

    let mut player = Player::new (
        3.456,
        2.345,
        1.523,
        std::f32::consts::PI / 3.,
    );

    let map = match Map::init(16, 16) {
        Ok(m) => m,
        Err(_) => {
            panic!("Could not open map");
        }
    };

    let tex_walls = Texture::new("./walltex.png").expect("Could not open wall texture");
    let tex_monsters = Texture::new("./monsters.png").expect("Could not open monster texture");
    let mut sprites = vec![
        Sprite::new(3.523, 3.812, 2, 0.0),
        Sprite::new(1.834, 8.765, 0, 0.0),
        Sprite::new(5.323, 5.365, 1, 0.0),
        Sprite::new(4.123, 10.265, 1, 0.0),
    ];

    for frame in 0..360 {
    // for frame in 0..5 {
    // for frame in 0..1 {
        let output_path = "./out/";
        let ss = format!("{}{:05}.ppm", output_path, frame);
        // player.a -= 2. * std::f32::consts::PI / 360.;
        player.set_a( player.get_a() - (2. * std::f32::consts::PI / 360.) );
        render(&mut fb, &map, &player, &mut sprites, &tex_walls, &tex_monsters).expect("Could not render image");
        utils::drop_ppm_image(ss.as_str(), &fb.img, fb.w as usize, fb.h as usize)
            .expect("Could not drop image");
    }

    println!("Rendered all frames, collecting into gif...");
    let output = Command::new("convert")
        .args(&["-delay", "10", "-loop", "0", "*.ppm", "rendered.gif"])
        .current_dir("out/")
        .output()
        .expect("Could not start process");

    println!("Status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    println!("done");

    //open results in Finder
    Command::new("open")
        .arg("out/")
        .output()
        .expect("Could not open folder");

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

        let (r, g, b, a) = utils::unpack_color(packed);

        assert_eq!(vec![2, 4, 8, 16], vec![r, g, b, a]);
    }

    #[test]
    fn packs_ints_idempotently() {
        let r = 2;
        let g = 4;
        let b = 8;
        let a = 255;

        let color = utils::pack_color_rgba(r, g, b, a);

        let (rc, gc, bc, ac) = utils::unpack_color(color);

        assert_eq!(vec![r, g, b, a], vec![rc, gc, bc, ac]);
    }
}
