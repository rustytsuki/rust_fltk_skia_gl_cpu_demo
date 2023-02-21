pub mod gl;
pub mod software;

use std::{fs::File, io::Write, time::Instant};

use fltk::window::Window;
use skia_safe::{Canvas, Surface, EncodedImageFormat};

pub struct FPS {
    start: Instant,
    count: f64,
    fps: f64,
}

impl FPS {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            count: 0.,
            fps: 0.
        }
    }

    pub fn update(&mut self) {
        let duration = self.start.elapsed().as_secs_f64();
        self.count += 1.;
        self.fps = self.count / duration;
        if duration > 2. {
            self.start = Instant::now();
            self.count = 0.;
        }
    }

    pub fn fps(&self) -> f64 {
        self.fps
    }
}
pub trait Device {
    fn update(&mut self);
    fn resize(&mut self);
    fn fps(&self) -> f64;
    fn info(&self) -> String;
    fn add_window(&mut self, window: &Window);
    fn surface(&mut self) -> &mut Surface;
    fn canvas(&mut self) -> &mut Canvas;
    fn flush(&mut self);
    fn save_png(&mut self, file: &str) {
        let image = self.surface().image_snapshot();
        let data = image.encode_to_data(EncodedImageFormat::PNG).unwrap();
        let mut file = File::create(file).unwrap();
        let bytes = data.as_bytes();
        file.write_all(bytes).unwrap();
    }
}