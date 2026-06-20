// #![feature(lock_value_accessors)]
#![allow(static_mut_refs)]
use std::{
    os::raw,
    thread::sleep,
    time::{Duration, Instant},
};

use embedded_graphics::{Drawable, draw_target::DrawTarget, pixelcolor::Rgb565, prelude::*};
use embedded_graphics_doom::{ScreenBuffer, colors, create, tick};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

const WINDOW_X: usize = 256;
const WINDOW_Y: usize = 64;

const X: usize = 320;
const Y: usize = 200;
const SIZE: usize = X * Y;

pub static mut DISPLAY: Option<SimulatorDisplay<Rgb565>> = None;
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
    display.clear(Rgb565::BLACK).unwrap();

    const SCALE_FACTOR: f32 = (WINDOW_X as f32 / X as f32).min(WINDOW_Y as f32 / Y as f32);
    const SCALED_RENDER_WIDTH: usize = (X as f32 * SCALE_FACTOR) as usize;
    const SCALED_RENDER_HEIGHT: usize = (Y as f32 * SCALE_FACTOR) as usize;

    let scale_fn = |n: usize| -> usize { (n as f32 / SCALE_FACTOR) as usize };

    for y in 0..SCALED_RENDER_HEIGHT {
        for x in 0..SCALED_RENDER_WIDTH {
            let idx = scale_fn(y) * X + scale_fn(x);
            // println!("{},{}  -->  {},{}", x, y, scale_fn(x), scale_fn(y));
            let color = palette565[buf[idx] as usize];

            Pixel(Point::new(x as i32, y as i32), color)
                .draw(display)
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
    let output_settings = OutputSettingsBuilder::new().scale(6).build();
    let mut window = Window::new("Doom Embedded_Graphics", &output_settings);

    unsafe { START = Some(Instant::now()) };
    let display = SimulatorDisplay::<Rgb565>::new(Size::new(WINDOW_X as u32, WINDOW_Y as u32));
    unsafe { DISPLAY = Some(display) };

    unsafe { create(&SCREEN_BUFFER) };

    loop {
        tick();
        window.update(unsafe { &DISPLAY.as_ref().unwrap() });
        if window.events().any(|e| e == SimulatorEvent::Quit) {
            break;
        }
    }
}
