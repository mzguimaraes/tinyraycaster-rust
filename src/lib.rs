pub struct Texture {
    pub img_w: u32,
    pub img_h: u32,
    pub count: u32,
    pub size: u32,
    img: Vec<u32>,
}

impl Texture {
    pub fn new(filename: &str) -> Result<Texture, image::ImageError> {
        let pixmap = image::open(filename)?
            .as_rgba8()
            .expect("Cannot open texture")
            .to_owned();

        let (w, h) = pixmap.dimensions();
        let ntextures = w / h;

        let size = w / ntextures;

        let pixmap: Vec<u32> = pixmap
            .to_vec()
            .chunks(4)
            .map(|x| utils::pack_color_rgba(x[0], x[1], x[2], x[3]))
            .collect();

        Ok(Texture {
            img_w: w,
            img_h: h,
            count: ntextures,
            size,
            img: pixmap,
        })
    }

    //get pixel (i,j) from texture idx
    pub fn get(&self, i: u32, j: u32, idx: u32) -> Option<u32> {
        self.img
            .get((i + idx * self.size + j * self.img_w) as usize)
            .cloned()
    }

    // retrieve one column (tex_coord) from the texture texture_id and scale it to the desired size
    pub fn get_scaled_column(
        &self,
        texture_id: u32,
        tex_coord: u32,
        column_height: u32,
    ) -> Option<Vec<u32>> {
        let mut column: Vec<u32> = vec![0; column_height as usize];
        for y in 0..column_height {
            column[y as usize] =
                match self.get(tex_coord, (y * self.size) / column_height, texture_id) {
                    Some(pix) => pix,
                    None => return None,
                }
        }
        Some(column)
    }
}

pub struct Map {
    layout: Vec<char>,
    pub w: u32,
    pub h: u32,
}

pub enum MapError {
    BadParameters,
}

impl Map {
    pub fn init(width: u32, height: u32) -> Result<Map, MapError> {
        let layout: Vec<char> = "0000222222220000\
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
            .collect();
        if width * height == layout.len() as u32 {
            Ok(Map {
                layout,
                w: width,
                h: height,
            })
        } else {
            Err(MapError::BadParameters)
        }
    }

    pub fn get(&self, i: u32, j: u32) -> Option<u32> {
        self.layout.get((i + j * self.w) as usize)?.to_digit(10)
    }

    pub fn is_empty(&self, i: u32, j: u32) -> bool {
        self.layout[(i + j * self.w) as usize] == ' '
    }
}

#[derive(Debug)]
pub enum FrameError {
    PixelOutOfBounds,
}

extern crate sdl2;
use sdl2::render::WindowCanvas;
use sdl2::pixels;
use sdl2::gfx::primitives::DrawRenderer;

pub struct Framebuffer<'a> {
    pub w: u32,
    pub h: u32,
    // pub img: Vec<u32>,
    pub img: &'a mut WindowCanvas,
}

impl<'a> Framebuffer<'a> {
    pub fn new(canvas: &'a mut WindowCanvas, width: u32, height: u32) -> Framebuffer<'a> {
        Framebuffer {
            w: width,
            h: height,
            // img: vec![utils::pack_color_rgb(255, 255, 255); (width * height) as usize],
            img: canvas,
        }
    }

    pub fn clear(&mut self, color: u32) {
        // self.img = vec![color; (self.w * self.h) as usize];
        self.img.set_draw_color(pixels::Color::from(utils::unpack_color(color)));
        self.img.clear();
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) -> Result<(), String> {
        // match self.img.get_mut((x + y * self.w) as usize) {
        //     Some(pix) => {
        //         *pix = color;
        //         Ok(())
        //     }
        //     None => Err(FrameError::PixelOutOfBounds),
        // }
        self.img.pixel(x as i16, y as i16, pixels::Color::from(utils::unpack_color(color)))
    }

    pub fn present(&mut self) {
        self.img.present();
    }

    pub fn draw_rectangle(
        &mut self,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        color: u32,
    ) -> Result<(), String> {
        for i in 0..w {
            for j in 0..h {
                let cx = x + i;
                let cy = y + j;
                if cx < self.w && cy < self.h {
                    self.set_pixel(cx, cy, color)?;
                }
            }
        }
        Ok(())
    }
}

pub struct Player {
    pub x: f32,
    pub y: f32,
    a: f32,
    pub fov: f32,
}

impl Player {
    pub fn new(x: f32, y: f32, a: f32, fov: f32) -> Player {
        Player {
            x: x,
            y: y,
            a: a,
            fov: fov,
        }
    }

    pub fn get_a(&self) -> f32 { self.a }

    pub fn set_a(&mut self, val: f32) {
        let mut new_val = val.clone();
        while new_val > 2. * std::f32::consts::PI { new_val -= 2. * std::f32::consts::PI; }
        while new_val < 0. { new_val += 2. * std::f32::consts::PI; }
        self.a = new_val;
    }
}

pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub tex_id: u32,
    pub player_dist: f32,
}

impl Sprite {
    pub fn new(x: f32, y: f32, tex_id: u32, player_dist: f32) -> Sprite {
        Sprite { x, y, tex_id, player_dist }
    }
}

pub mod utils {
    use std::fs;
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use std::path::Path;

    pub fn pack_color_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
        let b1 = u32::from(r);
        let b2 = u32::from(g);
        let b3 = u32::from(b);
        let b4 = u32::from(a);
        (b4 << 24) + (b3 << 16) + (b2 << 8) + b1
    }

    // rust does not have function overloading
    pub fn pack_color_rgb(r: u8, g: u8, b: u8) -> u32 {
        pack_color_rgba(r, g, b, 255)
    }

    pub fn unpack_color(color: u32) -> (u8, u8, u8, u8) {
        let r = (color & 255) as u8; //keep last 8 bits
        let g = (color.rotate_right(8) & 255) as u8;
        let b = (color.rotate_right(16) & 255) as u8;
        let a = (color.rotate_right(24) & 255) as u8;
        return (r, g, b, a);
    }

    pub fn drop_ppm_image(
        filename: &str,
        image: &[u32],
        w: usize,
        h: usize,
    ) -> std::io::Result<()> {
        assert_eq!(image.len(), w * h);
        if Path::new(filename).exists() {
            fs::remove_file(filename)?;
        }
        let output = File::create(filename).expect("Cannot open or create file");
        let mut output = BufWriter::new(output);
        let header = format!("P6\n{} {}\n255\n", w, h);

        output
            .write_all(header.as_bytes())
            .expect("cannot write header");

        for pixel in image.iter().take(w * h) {
            let (r, g, b, _) = unpack_color(*pixel);

            output.write_all(&[r, g, b])?;
        }
        //output closes at end of scope
        println!("Wrote image {}", filename);
        Ok(())
    }
}
