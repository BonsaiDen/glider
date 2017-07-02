// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Crates ---------------------------------------------------------------------
extern crate gfx;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate glutin;
extern crate image;



// STD Dependencies -----------------------------------------------------------
use std::thread;
use std::time::{Instant, Duration};


// External Dependencies ------------------------------------------------------
use gfx::Device;
use glutin::{
    Event as InputEvent,
    EventsLoop,
    ElementState,
    WindowBuilder, WindowEvent
};


// Internal Dependencies ------------------------------------------------------
mod input;
mod texture;

use input::{ButtonState, KeyState};

pub use input::{Key, Keyboard, Button, Mouse};
pub use texture::Texture;


// Type Abstractions ----------------------------------------------------------
pub type ColorBuffer = gfx::handle::RenderTargetView<gfx_device_gl::Resources, (gfx::format::R8_G8_B8_A8, gfx::format::Srgb)>;
pub type DepthBuffer = gfx::handle::DepthStencilView<gfx_device_gl::Resources, (gfx::format::D24_S8, gfx::format::Unorm)>;
pub type Encoder = gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>;
pub type Factory = gfx_device_gl::Factory;

// Traits ---------------------------------------------------------------------
pub trait Renderable {
    fn draw(
        &mut self,
        time: f32,
        dt: f32,
        encoder: &mut Encoder,
        &Keyboard,
        &Mouse,
        Option<((u32, u32), ColorBuffer, DepthBuffer)>

    ) where Self: Sized;
}

pub struct RenderTarget {
    pub factory: Factory,
    pub width: u32,
    pub height: u32,
    pub color: ColorBuffer,
    pub depth: DepthBuffer
}

// Public Interface -----------------------------------------------------------
pub fn run<
    R,
    C: FnOnce(RenderTarget) -> R
>(
    title: &str,
    width: u32,
    height: u32,
    fps: u32,
    callback: C

) where R: Renderable {

    let builder = WindowBuilder::new()
        .with_title(title.to_string())
        .with_dimensions(width, height)
        //.with_multisampling(4);
        //.with_min_dimensions(width, height)
        //.with_max_dimensions(width, height)
        .with_vsync();

    let events = EventsLoop::new();
    let (
        window,
        mut device,
        mut factory,
        mut output_color,
        mut output_depth

    ) = gfx_window_glutin::init::<
        gfx::format::Srgba8,
        gfx::format::DepthStencil

    >(builder, &events);

    println!("[Renderer] Window created");

    let frame_time = Duration::new(0, 1000000000 / fps);

    let mut encoder: gfx::Encoder<
        gfx_device_gl::Resources,
        gfx_device_gl::CommandBuffer

    > = factory.create_command_buffer().into();

    let mut renderable = {
        let refs = RenderTarget {
            factory: factory,
            width: width,
            height: height,
            color: output_color.clone(),
            depth: output_depth.clone()
        };
        callback(refs)
    };

    let mut mouse_pos = (-1, -1);
    let mut keyboard = Keyboard::new(48, ());
    let mut mouse = Mouse::new(2, mouse_pos);

    let mut running = true;
    let mut time: f32 = 0.0;
    let mut dt: f32 = 0.0;

    println!("[Renderer] Mainloop started");
    while running {

        let started = Instant::now();

        keyboard.advance();
        mouse.advance();

        let mut resized = None;
        events.poll_events(|event| {
            match event {
                InputEvent::WindowEvent{ event: WindowEvent::Closed, .. } => {
                    running = false;
                },
                InputEvent::WindowEvent{ event: WindowEvent::Focused(_), .. } => {
                    keyboard.reset();
                    mouse.reset();
                    mouse_pos = (-1, -1);
                },
                InputEvent::WindowEvent{ event: WindowEvent::MouseMoved(x, y), .. } => {
                    mouse_pos = (x, y);
                    mouse.set_position(mouse_pos);
                },
                InputEvent::WindowEvent{ event: WindowEvent::MouseInput(ElementState::Pressed, button), .. } => {
                    if mouse_pos.0 != -1 || mouse_pos.1 != -1 {
                        mouse.set(button.into(), ButtonState::WasPressed(mouse_pos.0, mouse_pos.1));
                    }
                },
                InputEvent::WindowEvent{ event: WindowEvent::MouseInput(ElementState::Released, button), .. } => {
                    if mouse_pos.0 != -1 || mouse_pos.1 != -1 {
                        mouse.set(button.into(), ButtonState::WasReleased(mouse_pos.0, mouse_pos.1));
                    }
                },
                InputEvent::WindowEvent{ event: WindowEvent::KeyboardInput(ElementState::Pressed, _, Some(key), _), .. } => {
                    keyboard.set(key.into(), KeyState::WasPressed);
                },
                InputEvent::WindowEvent{ event: WindowEvent::KeyboardInput(ElementState::Released, _, Some(key), _), .. } => {
                    keyboard.set(key.into(), KeyState::WasReleased);
                },
                InputEvent::WindowEvent{ event: WindowEvent::Resized(w, h), .. } => {
                    gfx_window_glutin::update_views(
                        &window,
                        &mut output_color,
                        &mut output_depth
                    );
                    resized = Some(((w, h), output_color.clone(), output_depth.clone()));
                },
                _ => {}
            }
        });

        // Draw
        encoder.clear_depth(&output_depth, 1.0);
        encoder.clear(&output_color, [0.0, 0.0, 0.0, 1.0]);
        renderable.draw(time, dt, &mut encoder, &keyboard, &mouse, resized);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();

        // Limit FPS
        let remaining = started.elapsed();
        if remaining < frame_time {
            thread::sleep(frame_time - remaining);

        } else {
            println!("Exceeded frame time: {:?}", started.elapsed());
        }

        let e = started.elapsed();
        dt = e.as_secs() as f32 + e.subsec_nanos() as f32 / 1000000000.0;
        time += dt;

    }

    println!("[Renderer] Mainloop ended");

}

