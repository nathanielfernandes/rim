use image::{imageops, DynamicImage};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Ops {
    fliph: Option<bool>,
    flipv: Option<bool>,
    scale: Option<f32>,
    blur: Option<f32>,
}

impl Ops {
    pub fn exec(&self, mut img: DynamicImage) -> DynamicImage {
        let scale = self.scale.unwrap_or(1.0).max(0.1).min(3.0);

        if scale < 1.0 {
            img = Self::_scale(img, scale);
            img = self._exec(img);
        } else if scale == 1.0 {
            img = self._exec(img);
        } else {
            img = self._exec(img);
            img = Self::_scale(img, scale);
        }

        img
    }

    pub fn _exec(&self, mut img: DynamicImage) -> DynamicImage {
        if self.fliph.unwrap_or(false) {
            img = img.fliph()
        }

        if self.flipv.unwrap_or(false) {
            img = img.flipv()
        }

        if let Some(sigma) = self.blur {
            if sigma > 0.1 {
                img =
                    image::DynamicImage::ImageRgba8(imageops::blur(&img, sigma.max(0.2).min(20.0)));
            }
        }

        return img;
    }

    pub fn _scale(img: DynamicImage, scale: f32) -> DynamicImage {
        let width = img.width();
        let height = img.height();

        image::DynamicImage::ImageRgba8(imageops::resize(
            &img,
            (width as f32 * scale).round() as u32,
            (height as f32 * scale).round() as u32,
            imageops::FilterType::Lanczos3,
        ))
    }
}
