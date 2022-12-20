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
    time::{Duration, Instant},
};

use runner::{CpuRunner, GpuRunner};

use forma::prelude::*;
use winit::{
    dpi::PhysicalPosition,
    event::{
        ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode,
        WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
};

pub struct RunContext<'a> {
    elapsed: Duration,
    keyboard: &'a Keyboard,
    mouse: &'a Mouse,
}

trait App {
    fn set_width(&mut self, width: usize);
    fn set_height(&mut self, height: usize);
    fn update<'a>(&mut self, context: &RunContext<'a>);
    fn compose<'a>(&mut self, composition: &mut Composition, context: &RunContext<'a>);
}

trait Runner {
    fn resize(&mut self, width: u32, height: u32);
    fn render<'a>(&mut self, app: &mut dyn App, context: RunContext<'a>);
}

struct Mouse {
    pressed: HashSet<MouseButton>,
    wheel: Option<PhysicalPosition<f64>>,
    position: PhysicalPosition<f64>,
    position_delta: PhysicalPosition<f64>,
}

impl Mouse {
    fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            wheel: None,
            position: PhysicalPosition::default(),
            position_delta: PhysicalPosition::default(),
        }
    }

    fn pressed_left(&self) -> bool {
        self.pressed.contains(&winit::event::MouseButton::Left)
    }

    fn on_mouse_input(&mut self, button: winit::event::MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => self.pressed.insert(button),
            ElementState::Released => self.pressed.remove(&button),
        };
    }

    fn update_position(&mut self, position: PhysicalPosition<f64>) {
        self.position_delta = PhysicalPosition {
            x: self.position.x - position.x,
            y: self.position.y - position.y,
        };
        self.position = position;
    }

    fn update_wheel(&mut self, wheel: PhysicalPosition<f64>) {
        self.wheel = Some(wheel);
    }

    fn clear(&mut self) {
        self.wheel = None;
    }
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

fn main() {
    let width = 1024.;
    let height = 768.;

    let event_loop = EventLoop::new();
    let device = 0; // 0 CPU, 1 Low GPU, 2: High GPU
    let mut runner: Box<dyn Runner> = match device {
        0 => Box::new(CpuRunner::new(&event_loop, width as u32, height as u32)),
        1 => Box::new(GpuRunner::new(
            &event_loop,
            width as u32,
            height as u32,
            wgpu::PowerPreference::LowPower,
        )),
        _ => Box::new(GpuRunner::new(
            &event_loop,
            width as u32,
            height as u32,
            wgpu::PowerPreference::HighPerformance,
        )),
    };

    let mut app = draw::Drawer::new();

    let mut instant = Instant::now();
    let mut keyboard = Keyboard::new();
    let mut mouse = Mouse::new();

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
                    WindowEvent::MouseWheel {
                        delta: MouseScrollDelta::PixelDelta(delta),
                        ..
                    },
                ..
            } => {
                mouse.update_wheel(delta);
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                mouse.update_position(position);
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                mouse.on_mouse_input(button, state);
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

                let context = RunContext {
                    elapsed,
                    keyboard: &keyboard,
                    mouse: &mouse,
                };

                runner.render(&mut app, context);

                mouse.clear();
            }
            _ => (),
        }
    });
}
