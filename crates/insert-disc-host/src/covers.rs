//! Pipeline de capa (SECURITY R6): valida por **conteúdo**, limita tamanho e dimensões, recusa
//! SVG e qualquer formato que não seja PNG/JPEG/WebP, reencoda como JPEG e calcula a cor da
//! lombada. Toda capa, de qualquer origem, passa por aqui antes de ser salva nos dados do app.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat, ImageReader, RgbImage};

pub const MAX_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_SIDE: u32 = 8192;
/// Tamanho salvo (retrato 2:3 da Steam); imagens maiores são reduzidas.
const OUT_W: u32 = 600;
const OUT_H: u32 = 900;
/// Cor do texto da lombada (`--etiqueta` em ui/css/tokens.css).
const LABEL_RGB: [u8; 3] = [0xf3, 0xf5, 0xf4];
const MIN_CONTRAST: f64 = 4.5;

#[derive(Debug, PartialEq, Eq)]
pub enum CoverError {
    TooBig,
    NotAnImage,
    Dimensions,
    Decode,
}

pub struct Cover {
    pub jpeg: Vec<u8>,
    /// `#rrggbb`, escurecida até ter contraste ≥ 4,5:1 com o texto da lombada.
    pub spine: String,
}

fn luminance([r, g, b]: [u8; 3]) -> f64 {
    let lin = |c: u8| {
        let c = c as f64 / 255.0;
        if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
}

pub fn contrast(a: [u8; 3], b: [u8; 3]) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Média da imagem reduzida, escurecida em passos de 8% até o contraste mínimo com a etiqueta.
pub fn spine_color(img: &RgbImage) -> String {
    let small = image::imageops::thumbnail(img, 16, 16);
    let n = (small.width() * small.height()).max(1) as u64;
    let sum = small.pixels().fold([0u64; 3], |s, p| [s[0] + p[0] as u64, s[1] + p[1] as u64, s[2] + p[2] as u64]);
    let mut c = [(sum[0] / n) as u8, (sum[1] / n) as u8, (sum[2] / n) as u8];
    while contrast(c, LABEL_RGB) < MIN_CONTRAST {
        c = c.map(|v| (v as f64 * 0.92) as u8);
    }
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

pub fn process(bytes: &[u8]) -> Result<Cover, CoverError> {
    if bytes.len() > MAX_BYTES {
        return Err(CoverError::TooBig);
    }
    // formato pelo conteúdo (magic bytes), nunca pela extensão; SVG, GIF, BMP etc. caem aqui
    let fmt = match image::guess_format(bytes) {
        Ok(f @ (ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP)) => f,
        _ => return Err(CoverError::NotAnImage),
    };
    let (w, h) = ImageReader::with_format(Cursor::new(bytes), fmt).into_dimensions().map_err(|_| CoverError::Decode)?;
    if w == 0 || h == 0 || w > MAX_SIDE || h > MAX_SIDE {
        return Err(CoverError::Dimensions);
    }
    let mut img = image::load_from_memory_with_format(bytes, fmt).map_err(|_| CoverError::Decode)?;
    if w > OUT_W || h > OUT_H {
        img = img.resize(OUT_W, OUT_H, FilterType::Triangle);
    }
    let rgb = DynamicImage::ImageRgb8(img.to_rgb8()).to_rgb8();
    let mut jpeg = Vec::new();
    JpegEncoder::new_with_quality(&mut jpeg, 85).encode_image(&rgb).map_err(|_| CoverError::Decode)?;
    Ok(Cover { spine: spine_color(&rgb), jpeg })
}

/// Nome de arquivo válido de capa salva (`<id>.jpg`, só hexadecimal e hífen): impede `..` e caminhos.
pub fn valid_name(name: &str) -> bool {
    name.strip_suffix(".jpg").is_some_and(|s| !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-'))
}

pub fn file_in(dir: &Path, name: &str) -> Option<PathBuf> {
    valid_name(name).then(|| dir.join(name)).filter(|p| p.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};

    fn png(w: u32, h: u32, px: [u8; 3]) -> Vec<u8> {
        let img: RgbImage = ImageBuffer::from_pixel(w, h, Rgb(px));
        let mut out = Cursor::new(Vec::new());
        img.write_to(&mut out, ImageFormat::Png).unwrap();
        out.into_inner()
    }

    #[test]
    fn valid_png_is_reencoded_as_jpeg_and_shrunk() {
        let c = process(&png(1200, 1800, [200, 30, 30])).unwrap();
        assert_eq!(image::guess_format(&c.jpeg).unwrap(), ImageFormat::Jpeg);
        let (w, h) = ImageReader::with_format(Cursor::new(&c.jpeg), ImageFormat::Jpeg).into_dimensions().unwrap();
        assert!(w <= OUT_W && h <= OUT_H, "{w}x{h}");
    }

    #[test]
    fn fakes_svg_and_garbage_are_refused_by_content() {
        assert_eq!(process(b"<svg xmlns='http://www.w3.org/2000/svg'><script>alert(1)</script></svg>").err(), Some(CoverError::NotAnImage));
        assert_eq!(process(b"isto nao e uma imagem.png").err(), Some(CoverError::NotAnImage));
        assert_eq!(process(b"GIF89a\x01\x00\x01\x00").err(), Some(CoverError::NotAnImage));
        assert_eq!(process(&[]).err(), Some(CoverError::NotAnImage));
        // cabeçalho de PNG válido, corpo truncado: erro, nunca pânico
        let mut cut = png(50, 50, [1, 2, 3]);
        cut.truncate(40);
        assert!(process(&cut).is_err());
    }

    #[test]
    fn oversize_files_and_dimensions_are_refused() {
        assert_eq!(process(&vec![0u8; MAX_BYTES + 1]).err(), Some(CoverError::TooBig));
        assert_eq!(process(&png(MAX_SIDE + 1, 2, [0, 0, 0])).err(), Some(CoverError::Dimensions));
    }

    #[test]
    fn spine_color_reaches_the_minimum_contrast_for_light_and_dark_covers() {
        for px in [[255, 255, 255], [250, 240, 120], [0, 0, 0], [30, 60, 200], [240, 30, 30]] {
            let c = process(&png(64, 96, px)).unwrap().spine;
            let rgb = [1, 3, 5].map(|i| u8::from_str_radix(&c[i..i + 2], 16).unwrap());
            assert!(contrast(rgb, LABEL_RGB) >= MIN_CONTRAST, "{px:?} -> {c}");
        }
        // capa escura fica parecida com ela mesma (não é clareada)
        assert_eq!(process(&png(64, 96, [0, 0, 0])).unwrap().spine, "#000000");
    }

    #[test]
    fn stored_names_cannot_escape_the_covers_folder() {
        assert!(valid_name("3f2b7c1e-0a4d-4f5b-9c1d-2e8a7b6c5d4e.jpg"));
        for bad in ["../x.jpg", "a/b.jpg", "a\\b.jpg", ".jpg", "x.png", "x.jpg.exe", "", "g.jpg", "..\\..\\Windows\\win.ini"] {
            assert!(!valid_name(bad), "{bad}");
        }
    }
}
