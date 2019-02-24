pub mod textures {
    extern crate image;
    use crate::utils;
    pub struct Texture {
        img_w: u32,
        img_h: u32,
        count: u32,
        size: usize,
        img: Box<Vec<u32>>,
    }

    impl Texture {
        pub fn new(filename: &str) -> Result<Texture, image::ImageError> {
            let pixmap = image::open(filename)?
                .as_rgba8()
                .expect("Cannot open texture")
                .to_owned();

            let (w, h) = pixmap.dimensions();
            let ntextures = w / h;

            let size = (w / ntextures) as usize;

            let pixmap: Vec<u32> = pixmap
                .to_vec()
                .chunks(4)
                .map(|x| utils::pack_color_rgba(x[0], x[1], x[2], x[3]))
                .collect();

            Ok(Texture {
                img_w: w,
                img_h: h,
                count: ntextures,
                size: size,
                img: Box::new(pixmap),
            })
        }

        //get pixel (i,j) from texture idx
        pub fn get(&self, i: usize, j: usize, idx: usize) -> Option<u32> {
            self.img
                .get(i + idx * self.size + j * self.img_w as usize)
                .cloned()
        }

        // retrieve one column (tex_coord) from the texture texture_id and scale it to the desired size
        pub fn get_scaled_column(
            &self,
            texture_id: usize,
            tex_coord: usize,
            column_height: usize,
        ) -> Option<Vec<u32>> {
            let mut column: Vec<u32> = vec![0; column_height];
            for y in 0..column_height {
                column[y] = match self.get(tex_coord, (y * self.size) / column_height, texture_id) {
                    Some(pix) => pix,
                    None => return None,
                }
            }
            Some(column)
        }
    }
}

pub mod map {
    pub struct Map {
        layout: Vec<char>,
        w: u32,
        h: u32,
    }

    impl Map {
        pub fn init(width: u32, height: u32) -> Result<Map, String> {
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
            match width * height == layout.len() as u32 {
                true => Ok(Map {
                    layout: layout,
                    w: width,
                    h: height,
                }),
                false => Err(String::from(
                    "width * height must equal dimensions of layout",
                )),
            }
        }

        pub fn get(&self, i: usize, j: usize) -> Option<u32> {
            self.layout.get(i + j * self.w as usize)?.to_digit(10)
        }

        pub fn is_empty(&self, i: usize, j: usize) -> bool {
            self.layout.get(i + j * self.w as usize).unwrap().to_owned() == ' '
        }
    }
}

pub mod framebuffer {
    pub struct Framebuffer {
        w: usize,
        h: usize,
        img: Vec<u32>,
    }

    pub enum FrameError {
        PixelOutOfBounds,
    }

    impl Framebuffer {

        pub fn clear(&mut self, color: u32) {
            self.img = vec![color; self.w * self.h];
        }

        pub fn set_pixel(&mut self, x: usize, y: usize, color: u32) -> Result<(), FrameError> {
            match self.img.get_mut(x + y * self.w) {
                Some(pix) => {
                    *pix = color;
                    Ok(())
                }
                None => Err(FrameError::PixelOutOfBounds),
            }
        }

        pub fn draw_rectangle(
            &mut self,
            x: usize,
            y: usize,
            w: usize,
            h: usize,
            color: u32,
        ) -> Result<(), FrameError> {
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
}

pub mod player {
    struct Player {
        x: f32,
        y: f32,
        a: f32,
        fov: f32,
    }
}

pub mod utils {
    use std::fs;
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use std::path::Path;

    pub fn pack_color_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
        let b1 = r as u32;
        let b2 = g as u32;
        let b3 = b as u32;
        let b4 = a as u32;
        (b4 << 24) + (b3 << 16) + (b2 << 8) + b1
    }

    //rust does not have function overloading
    pub fn pack_color_rgb(r: u8, g: u8, b: u8) -> u32 {
        pack_color_rgba(r, g, b, 255)
    }

    pub fn unpack_color(color: &u32, r: &mut u8, g: &mut u8, b: &mut u8, a: &mut u8) {
        *r = (color & 255) as u8; //keep last 8 bits
        *g = (color.rotate_right(8) & 255) as u8;
        *b = (color.rotate_right(16) & 255) as u8;
        *a = (color.rotate_right(24) & 255) as u8;
    }

    pub fn drop_ppm_image(
        filename: &str,
        image: &Vec<u32>,
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
        println!("Wrote image {}", filename);
        Ok(())
    }
}
