mod conversion;
mod draw;
mod helpers;
mod layout_types;
mod rich_text;
mod runner;
mod text;
mod types;

use std::{
    collections::HashSet,
    fmt,
    path::PathBuf,
    str::FromStr,
    time::{Duration, Instant},
};

use runner::{CpuRunner, GpuRunner};

use forma::prelude::*;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

trait App {
    fn set_width(&mut self, width: usize);
    fn set_height(&mut self, height: usize);
    fn compose(&mut self, composition: &mut Composition, elapsed: Duration, keyboard: &Keyboard);
}

trait Runner {
    fn resize(&mut self, width: u32, height: u32);
    fn render(&mut self, app: &mut dyn App, elapsed: Duration, keyboard: &Keyboard);
}

struct Keyboard {
    pressed: HashSet<VirtualKeyCode>,
}

impl Keyboard {
    fn new() -> Self {
        Self {
            pressed: HashSet::new(),
        }
    }

    fn is_key_down(&self, key: VirtualKeyCode) -> bool {
        self.pressed.contains(&key)
    }

    fn on_keyboard_input(&mut self, input: winit::event::KeyboardInput) {
        if let Some(code) = input.virtual_keycode {
            match input.state {
                ElementState::Pressed => self.pressed.insert(code),
                ElementState::Released => self.pressed.remove(&code),
            };
        }
    }
}

pub fn to_linear(rgb: [u8; 3]) -> Color {
    fn conv(l: u8) -> f32 {
        let l = f32::from(l) * 255.0f32.recip();

        if l <= 0.04045 {
            l * 12.92f32.recip()
        } else {
            ((l + 0.055) * 1.055f32.recip()).powf(2.4)
        }
    }

    Color {
        r: conv(rgb[0]),
        g: conv(rgb[1]),
        b: conv(rgb[2]),
        a: 1.0,
    }
}

fn main() {
    let width = 1024.;
    let height = 768.;

    let event_loop = EventLoop::new();
    // let mut runner: Box<dyn Runner> = match opts.device {
    //     Device::Cpu => Box::new(CpuRunner::new(&event_loop, width as u32, height as u32)),
    //     Device::GpuLowPower => Box::new(GpuRunner::new(
    //         &event_loop,
    //         width as u32,
    //         height as u32,
    //         wgpu::PowerPreference::LowPower,
    //     )),
    //     Device::GpuHighPerformance => Box::new(GpuRunner::new(
    //         &event_loop,
    //         width as u32,
    //         height as u32,
    //         wgpu::PowerPreference::HighPerformance,
    //     )),
    // };

    let mut runner = Box::new(GpuRunner::new(
        &event_loop,
        width as u32,
        height as u32,
        wgpu::PowerPreference::HighPerformance,
    ));

    // let mut runner = Box::new(CpuRunner::new(&event_loop, width as u32, height as u32));

    let mut app = draw::Drawer::new();

    let mut instant = Instant::now();
    let mut keyboard = Keyboard::new();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                keyboard.on_keyboard_input(input);
            }
            Event::WindowEvent {
                event:
                    WindowEvent::Resized(size)
                    | WindowEvent::ScaleFactorChanged {
                        new_inner_size: &mut size,
                        ..
                    },
                ..
            } => {
                runner.resize(size.width, size.height);

                app.set_width(size.width as usize);
                app.set_height(size.height as usize);
            }
            Event::MainEventsCleared => {
                let elapsed = instant.elapsed();
                instant = Instant::now();

                runner.render(&mut app, elapsed, &keyboard);
            }
            _ => (),
        }
    });
}
