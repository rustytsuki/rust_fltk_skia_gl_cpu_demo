use fltk::{
    enums::{Align, ColorDepth},
    prelude::*,
    window::{self, Window},
};
use skia_safe::{AlphaType, Canvas, ColorType, ImageInfo, Surface};

use super::{FPS, Device};


pub struct DeviceSoftware {
    fps: FPS,
    window: Window,
    surface: Surface,
}

impl DeviceSoftware {
    pub fn new(parent: &mut Window, x: i32, y: i32, w: i32, h: i32) -> Self {
        let mut window = window::Window::new(x, y, w, h, None).with_align(Align::Top);
        window.end();
        parent.add(&window);
        window.show();

        // create surface
        let surface = Self::create_surface(&mut window);
        Self { fps: FPS::new(), window, surface }
    }

    fn create_surface(window: &mut Window) -> Surface {
        let px_w = window.pixel_w();
        let px_h = window.pixel_h();
        let image_info = ImageInfo::new((px_w, px_h), ColorType::RGBA8888, AlphaType::Premul, None);
        let surface = Surface::new_raster(&image_info, None, None).unwrap();
        let mut surface_closure = surface.clone();
        window.draw(move |s| {
            let w = s.w();
            let h = s.h();
            let pixmap = surface_closure.peek_pixels().unwrap();
            let data = pixmap.bytes().unwrap();
            unsafe {
                if let Ok(mut img) = crate::image::RgbImage::from_data(data, px_w, px_h, ColorDepth::Rgba8) {
                    img.scale(w, h, false, true);
                    img.draw(0, 0, w, h);
                }
            }

            s.draw_children();
        });

        surface
    }
}

impl Device for DeviceSoftware {
    fn update(&mut self) {
        self.fps.update();
    }

    fn resize(&mut self) {
        let surface = Self::create_surface(&mut self.window);
        self.surface = surface;
    }

    fn fps(&self) -> f64 {
        self.fps.fps()
    }

    fn info(&self) -> String {
        let info = format!("Software FPS: {:.2}", self.fps());
        info
    }

    fn add_window(&mut self, window: &Window) {
        self.window.add(window);
    }

    fn surface(&mut self) -> &mut Surface {
        &mut self.surface
    }

    fn canvas(&mut self) -> &mut Canvas {
        self.surface.canvas()
    }

    fn flush(&mut self) {
        self.window.redraw();
    }
}
