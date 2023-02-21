use fltk::{
    enums::Align,
    prelude::*,
    window::{GlWindow, Window},
};
use gl_rs::types::GLint;
use skia_safe::{
    gpu::{gl::FramebufferInfo, BackendRenderTarget, SurfaceOrigin},
    Canvas, ColorType, Surface,
};

use super::{FPS, Device};

pub struct DeviceGL {
    fps: FPS,
    window: GlWindow,
    surface: Surface,
}

impl DeviceGL {
    pub fn new(parent: &mut Window, x: i32, y: i32, w: i32, h: i32) -> Option<Self> {
        // create window
        let mut window = GlWindow::new(x, y, w, h, None).with_align(Align::Top);
        window.end();
        parent.add(&window);
        window.show();

        // create surface
        gl_rs::load_with(|s| window.get_proc_address(s));
        if let Some(surface) = Self::create_surface(&mut window) {
            return Some(Self {
                fps: FPS::new(),
                window,
                surface
            });
        }

        fltk::app::delete_widget(window);
        None
    }

    fn create_surface(window: &mut GlWindow) -> Option<Surface> {
        if let Some(mut gr_context) = skia_safe::gpu::DirectContext::new_gl(None, None) {
            let fb_info = {
                let mut fboid: GLint = 0;
                unsafe { gl_rs::GetIntegerv(gl_rs::FRAMEBUFFER_BINDING, &mut fboid) };

                FramebufferInfo {
                    fboid: fboid.try_into().unwrap(),
                    format: skia_safe::gpu::gl::Format::RGBA8.into(),
                }
            };

            let stencil_bits = 8;
            let backend_render_target = BackendRenderTarget::new_gl((window.pixel_w() as i32, window.pixel_h() as i32), None, stencil_bits, fb_info);
            if let Some(surface) = Surface::from_backend_render_target(
                &mut gr_context,
                &backend_render_target,
                SurfaceOrigin::BottomLeft,
                ColorType::RGBA8888,
                None,
                None,
            ) {
                return Some(surface);
            }
        }

        None
    }
}

impl Device for DeviceGL {
    fn update(&mut self) {
        self.fps.update();
    }

    fn resize(&mut self) {
        if let Some(surface) = Self::create_surface(&mut self.window) {
            self.surface = surface;
        }
    }

    fn fps(&self) -> f64 {
        self.fps.fps()
    }

    fn info(&self) -> String {
        let info = format!("OpenGL FPS: {:.2}", self.fps());
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
        self.surface.flush();
        self.window.flush();
    }
}

