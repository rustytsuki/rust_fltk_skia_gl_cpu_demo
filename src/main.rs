#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod device;
mod menu_bar;

use device::gl::DeviceGL;
use device::software::DeviceSoftware;
use device::Device;
use fltk::*;
use fltk::{app, button::Button, prelude::*, utils::hex2rgb};
use lazy_static::lazy_static;
use menu_bar::{Message, MyMenu};
use skia_safe::font_style::{Slant, Weight, Width};
use skia_safe::{Canvas, Image, Paint, PaintCap, PaintStyle, Path, Rect};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Instant;

lazy_static! {
    static ref IMAGE_CACHE: Mutex<HashMap<String, Arc<Image>>> = Mutex::new(HashMap::new());
}

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 800;

#[tokio::main]
async fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Base);
    let (r, g, b) = hex2rgb(0xf0f0f0);
    app::background(r, g, b);
    app::set_selection_color(255, 0, 0);

    let (s, _r) = app::channel::<Message>();

    let mut win = window::Window::default().with_size(WINDOW_WIDTH, WINDOW_HEIGHT);
    win.make_resizable(true);
    win.children();

    // menu bar
    let menu_bar = MyMenu::new(&s);
    menu_bar.menu.find_item("&File/Save\t").unwrap().deactivate();

    // context menu
    let mut menu = menu::MenuButton::default()
        .size_of_parent()
        .center_of_parent()
        .with_type(menu::MenuButtonType::Popup3);
    menu.add_choice("1st menu item\t|2nd menu item\t|3rd menu item\t");
    menu.set_callback(|m| println!("{:?}", m.choice()));

    win.end();
    win.show();

    // device
    let device: Rc<RefCell<dyn Device>>;

    let margin = 10;
    let x = margin;
    let y = menu_bar.menu.h();
    let canvas_w = win.w() - margin * 2;
    let canvas_h = win.h() - margin - y;

    if !cfg!(feature = "force_cpu") {
        if let Some(device_gl) = DeviceGL::new(&mut win, x, y, canvas_w, canvas_h) {
            device = Rc::new(RefCell::new(device_gl));
        } else {
            device = Rc::new(RefCell::new(DeviceSoftware::new(&mut win, x, y, canvas_w, canvas_h)));
        }
    } else {
        device = Rc::new(RefCell::new(DeviceSoftware::new(&mut win, x, y, canvas_w, canvas_h)));
    }

    // resize
    let device_closure = device.clone();
    win.resize_callback(move |_s, _x: i32, _y: i32, _w: i32, _h: i32| {
        device_closure.borrow_mut().resize();
    });

    // button
    let mut bt_win = window::Window::new(0, 0, 80, 40, None);
    Button::new(0, 0, 80, 40, "push me!");
    bt_win.end();
    device.borrow_mut().add_window(&bt_win);
    bt_win.show();

    // paint
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    let px_ratio = win.pixels_per_unit();
    let style = skia_safe::FontStyle::new(Weight::BOLD, Width::NORMAL, Slant::Upright);
    let typeface = skia_safe::Typeface::from_name("Arial", style).unwrap();
    let font = skia_safe::Font::new(typeface, 25.0 * px_ratio);

    // main loop
    let start = Instant::now();
    app::add_idle3(move |_| {
        let mut dev = device.borrow_mut();
        dev.update();
        let info = dev.info();
        let canvas = dev.canvas();
        canvas.clear(skia_safe::Color::GRAY);

        // draw
        draw_image(r"image.png", canvas, &mut paint);
        draw(&start, canvas, &mut paint);
        // dev.save_png("fltk_skia_test.png");

        // draw fps
        paint.set_style(PaintStyle::Fill);
        paint.set_color(skia_safe::Color::WHITE);
        paint.set_stroke_width(1.0);
        let blob = skia_safe::TextBlob::from_str(&info, &font).unwrap();
        let canvas_info = canvas.image_info();
        let canvas_px_w = canvas_info.width() as f32;
        canvas.draw_text_blob(
            &blob,
            ((canvas_px_w / px_ratio as f32 - 50. * 6.) * px_ratio, 50. * px_ratio),
            &paint,
        );
        win.set_label(&info);

        // flush
        dev.flush();

        // sleeps are necessary when calling redraw in the event loop ?
        // app::sleep(0.016);
    });

    app.run().unwrap();
}

fn draw(start: &Instant, canvas: &mut Canvas, paint: &mut Paint) {
    let duration = start.elapsed().as_secs_f64();

    let radius = (duration * 100.0) as f32;
    let center = (duration * 100.0) as f32;

    let mut path = Path::new();
    path.move_to((center + radius, center));
    for i in 1..8 {
        let a = 2.6927937 * i as f32;
        path.line_to((center + radius * a.cos(), center + radius * a.sin()));
    }

    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_cap(PaintCap::Round);
    let (r, g, b) = hex2rgb(0xb0bf1a);
    paint.set_color(skia_safe::Color::from_rgb(r, g, b));
    // paint.set_alpha(128);
    paint.set_stroke_width(6.0);
    canvas.draw_path(&path, paint);
}

fn draw_image(image_file: &str, canvas: &mut Canvas, paint: &mut Paint) {
    let mut image_cache = IMAGE_CACHE.lock().unwrap();
    if let Some(image) = image_cache.get(image_file) {
        draw_image_(image, canvas, paint);
    } else {
        let bin = include_bytes!("image.png");
        let data: skia_safe::Data;
        unsafe {
            data = skia_safe::Data::new_bytes(bin);
        }
        if let Some(image) = skia_safe::Image::from_encoded(data) {
            let image = Arc::new(image);
            draw_image_(&image, canvas, paint);
            image_cache.insert(String::from(image_file), image);
        } else {
            println!("error create image: {}", image_file);
        }
    }
}

fn draw_image_(image: &Arc<Image>, canvas: &mut Canvas, paint: &mut Paint) {
    let w = image.width() as f32;
    let h = image.height() as f32;
    canvas.draw_image_rect(&*image, None, Rect::from_xywh(100., 100., w, h), paint);
}
