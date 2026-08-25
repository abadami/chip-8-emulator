use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use crate::chip8::{VIDEO_SIZE, VIDEO_WIDTH};

pub fn draw_screen(canvas: &mut Canvas<Window>, display: &[u32; VIDEO_SIZE]) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in display.iter().enumerate() {
        if *pixel > 0 {
            let x = (i % VIDEO_WIDTH as usize) as i32;
            let y = (i / VIDEO_WIDTH as usize) as i32;

            let rect = Rect::new(x * 10, y * 10, 10, 10);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}
