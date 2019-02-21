pub mod framebuffer {
    
}

pub mod textures {
        extern crate image;
    pub struct Texture {
        img_w: u32,
        img_h: u32,
        count: u32,
        img: Box<Vec<u32>>
    }
    
    impl Texture {
        pub fn new(filename: &str) -> Result<Texture, image::ImageError> {
            let pixmap = image::open(filename)?
                .as_rgba8()
                .expect("Cannot open texture")
                .to_owned();

            let (w, h) = pixmap.dimensions();
            let ntextures = w / h;

            let pixmap = pixmap.to_vec()
                .into_iter()
                .map(|x| {

                })

            Ok(Texture {
                img_w: w,
                img_h: h,
                count: ntextures
            })
        }

        //get pixel (i,j) from texture idx
        pub fn get(&self, i: usize, j: usize, idx: usize) -> u32 {

        }

        // retrieve one column (tex_coord) from the texture texture_id and scale it to the destination size
        pub fn get_scaled_column(&self, texture_id: usize, tex_coord: usize, column_height: usize) -> Vec<u32> {

        } 
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

    pub fn drop_ppm_image(filename: &str, image: &Vec<u32>, w: usize, h: usize) -> std::io::Result<()> {
        assert_eq!(image.len(), w * h);
        if Path::new(filename).exists() {
            fs::remove_file(filename)?;
        }
        let output = File::create(filename).expect("Cannot open or create file");
        //let output = File::open(filename);
        //let output = match output {
        //Ok(file) => file,
        //Err(_) => File::create(filename).expect("Cannot open or create file"),
        //};
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