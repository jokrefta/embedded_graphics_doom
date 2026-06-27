// #![feature(lock_value_accessors)]
#![allow(static_mut_refs)]

mod config;

use std::{
    os::raw,
    thread::sleep,
    time::{Duration, Instant},
};

use embedded_graphics::{Drawable, draw_target::DrawTarget, pixelcolor::Rgb565, prelude::*};
use embedded_graphics_doom::{ScreenBuffer, colors, create, tick};
use rpi_led_panel::Canvas;

const WINDOW_X: usize = 256;
const WINDOW_Y: usize = 64;

const X: usize = 320;
const Y: usize = 200;
const SIZE: usize = X * Y;

pub static mut DISPLAY: Option<Box<Canvas>> = None;
pub static mut SCREEN_BUFFER: ScreenBuffer<X, Y, SIZE> = ScreenBuffer::new();
pub static mut START: Option<Instant> = None;

#[unsafe(no_mangle)]
extern "C" fn DG_DrawFrame() {
    let palette565: [Rgb565; 256] = unsafe {
        colors.map(|c| {
            Rgb565::new(
                ((c.r as u16 * 31) / 255) as u8, // red 5 bits
                ((c.g as u16 * 63) / 255) as u8, // green 6 bits
                ((c.b as u16 * 31) / 255) as u8, // blue 5 bits
            )
        })
    };
    let buf = unsafe { &SCREEN_BUFFER.0 };

    let display = unsafe { DISPLAY.as_mut().unwrap() };
    // display.clear(Rgb565::BLACK).unwrap();

    const SCALE_FACTOR: f32 = (WINDOW_X as f32 / X as f32).min(WINDOW_Y as f32 / Y as f32);
    const SCALED_RENDER_WIDTH: usize = (X as f32 * SCALE_FACTOR) as usize;
    const SCALED_RENDER_HEIGHT: usize = (Y as f32 * SCALE_FACTOR) as usize;

    let scale_fn = |n: usize| -> usize { (n as f32 / SCALE_FACTOR) as usize };

    for y in 0..SCALED_RENDER_HEIGHT {
        for x in 0..SCALED_RENDER_WIDTH {
            let idx = scale_fn(y) * X + scale_fn(x);
            // println!("{},{}  -->  {},{}", x, y, scale_fn(x), scale_fn(y));
            let color = palette565[buf[idx] as usize];

            Pixel(Point::new(x as i32, y as i32), color.into())
                .draw(display.as_mut())
                .unwrap();
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn DG_GetTicksMs() -> u32 {
    let start = unsafe { START.unwrap() };
    Instant::now()
        .duration_since(start)
        .as_millis()
        .try_into()
        .expect("Cannot Fit Start time into u32")
}

#[unsafe(no_mangle)]
extern "C" fn DG_GetKey(pressed: *mut raw::c_int, key: *mut raw::c_uchar) -> raw::c_int {
    0
}

#[unsafe(no_mangle)]
extern "C" fn DG_SleepMs(ms: u32) {
    sleep(Duration::from_millis(ms as u64));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <config filename>", &args[0]);
        return;
    }

    let config_parser = config::ConfigParser::from_file(args[1].clone()).unwrap();
    let led_config = config_parser.get_led_config().unwrap();
    let (mut matrix, canvas) =
        rpi_led_panel::RGBMatrix::new(led_config, 0).expect("Matrix initialization failed");

    assert_eq!(WINDOW_X, canvas.width());
    assert_eq!(WINDOW_Y, canvas.height());

    unsafe { START = Some(Instant::now()) };
    unsafe { DISPLAY = Some(canvas) };

    unsafe { create(&SCREEN_BUFFER) };

    loop {
        tick();
        unsafe {
            let canvas = std::mem::take(&mut DISPLAY);
            DISPLAY = Some(matrix.update_on_vsync(canvas.unwrap())); 
        }
    }
}
