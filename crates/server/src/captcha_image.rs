//! Self-contained image captcha generator. Renders a 4-char answer to a PNG
//! using the `image` crate. Mirrors `internal/captcha/image_captcha`.
use image::{ImageBuffer, Luma, Rgba, RgbaImage};
use rand::Rng;

const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// Generate a random answer string of `len` chars.
pub fn generate_len(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn generate() -> String {
    generate_len(4)
}

/// Render the answer as a PNG with light noise. Returns encoded PNG bytes.
pub fn render(answer: &str) -> Vec<u8> {
    let width = 120u32;
    let height = 40u32;
    let mut img: RgbaImage = ImageBuffer::new(width, height);

    // Background: light gray.
    for p in img.pixels_mut() {
        *p = Rgba([240, 240, 240, 255]);
    }

    // Draw each char with a pseudo-random offset (no font dependency: draw a
    // simple blocky glyph pattern keyed by the char code).
    let mut rng = rand::thread_rng();
    let char_w = width / (answer.len() as u32 + 1);
    for (i, ch) in answer.chars().enumerate() {
        let base_x = (i as u32 + 1) * char_w;
        let jitter = rng.gen_range(0..6u32);
        let base_y = 10u32 + jitter;
        // Glyph color.
        let color = Rgba([
            rng.gen_range(30..120u8),
            rng.gen_range(30..120u8),
            rng.gen_range(30..120u8),
            255,
        ]);
        draw_char(&mut img, ch, base_x, base_y, color);
    }

    // Add noise dots.
    for _ in 0..60 {
        let x = rng.gen_range(0..width);
        let y = rng.gen_range(0..height);
        img.put_pixel(
            x,
            y,
            Rgba([
                rng.gen_range(0..255u8),
                rng.gen_range(0..255u8),
                rng.gen_range(0..255u8),
                255,
            ]),
        );
    }

    let mut buf: Vec<u8> = Vec::new();
    let _ = img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png);
    buf
}

/// Draw a 5x7 blocky glyph. Each char maps to a distinct pattern via its code.
fn draw_char(img: &mut RgbaImage, ch: char, x0: u32, y0: u32, color: Rgba<u8>) {
    let code = (ch as u32) % 8;
    // 5 columns x 7 rows bitmap, pseudo-derived from code.
    for col in 0..5u32 {
        for row in 0..7u32 {
            let on = ((code.wrapping_add(col).wrapping_mul(31) + row * 7) % 3) != 0;
            if on {
                let x = x0 + col;
                let y = y0 + row;
                if x < img.width() && y < img.height() {
                    img.put_pixel(x, y, color);
                }
            }
        }
    }
}

/// Convert RGBA noise image helper (kept for parity with grayscale option).
#[allow(dead_code)]
fn to_luma(img: &RgbaImage) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        let p = img.get_pixel(x, y);
        Luma([((p[0] as u32 + p[1] as u32 + p[2] as u32) / 3) as u8])
    })
}
