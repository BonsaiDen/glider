// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Crates ---------------------------------------------------------------------
extern crate rand;
extern crate renderer;

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;

extern crate cgmath;
extern crate genmesh;
extern crate image;


// STD Dependencies -----------------------------------------------------------


// External Dependencies ------------------------------------------------------
use renderer::{Key, Keyboard, Mouse, Renderable, RenderTarget, Encoder, Factory, ColorBuffer, DepthBuffer};
use cgmath::{Vector3};

mod core;
mod render;

use self::core::{Camera, Course, Glider, Mesh};
use self::render::{LineView, MeshView};


// Game -----------------------------------------------------------------------
pub struct Game {
    factory: Factory,
    wireframe: bool,
    editing: bool,
    camera: Camera,
    course: Course,
    glider: Glider,

    editor_grid: Mesh,

    line_view: LineView,
    mesh_view: MeshView
}

impl Game {
    pub fn new(mut target: RenderTarget) -> Self {

        let line_view = LineView::new(
            &mut target.factory,
            target.color.clone(),
            target.depth.clone(),
            500
        );

        let mesh_view = MeshView::new(
            &mut target.factory,
            target.color.clone(),
            target.depth.clone()
        );

        Self {
            factory: target.factory,
            wireframe: false,
            editing: true,
            camera: Camera::new(target.width, target.height, 60.0),
            course: Course::new(),
            glider: Glider::new(),

            editor_grid: Mesh::from_grid_plane(10_000.0, 10_000.0, 100, 100),

            line_view: line_view,
            mesh_view: mesh_view
        }

    }
}

impl Renderable for Game {

    fn draw(
        &mut self,
        _: f32,
        dt: f32,
        mut encoder: &mut Encoder,
        keyboard: &Keyboard,
        _: &Mouse,
        resized: Option<((u32, u32), ColorBuffer, DepthBuffer)>

    ) where Self: Sized {

        if let Some(resized) = resized {

            self.mesh_view.resize(resized.clone());
            self.line_view.resize(resized.clone());

            let size = resized.0;
            self.camera.resize(size.0, size.1);

        }

        if keyboard.was_pressed(Key::B) {
            self.wireframe = !self.wireframe;
            self.mesh_view.reload(&mut self.factory, self.wireframe);
            self.line_view.reload(&mut self.factory, self.wireframe);
        }

        if keyboard.was_pressed(Key::R) {
            self.mesh_view.reload(&mut self.factory, self.wireframe);
            self.line_view.reload(&mut self.factory, self.wireframe);
            self.glider.set_position(self.course.start_point() + Vector3::new(10.0, 25.0, 0.0));
        }

        if keyboard.was_pressed(Key::Tab) {
            self.editing = !self.editing;
        }

        let view = if self.editing {

            // X-Axis
            self.line_view.add(Vector3::new(-5.0, 0.0, -5.0), Vector3::new(1000.0, 0.0, -5.0), [255.0, 0.0, 0.0, 1.0]);

            // Y-Axis
            self.line_view.add(Vector3::new(-5.0, 0.0, -5.0), Vector3::new(-5.0, 1000.0, -5.0), [0.0, 255.0, 0.0, 1.0]);

            // Z-Axis
            self.line_view.add(Vector3::new(-5.0, 0.0, -5.0), Vector3::new(-5.0, 0.0, 1000.0), [0.0, 0.0, 255.0, 1.0]);

            self.camera.update(&keyboard);
            self.course.edit(&keyboard);
            self.course.debug(&mut self.line_view);
            self.camera.view()

        } else {
            self.glider.update(dt, &self.course, &mut self.line_view, &keyboard);
            self.glider.camera_view()
        };

        self.glider.debug(&mut self.line_view);

        // Draw everything else
        self.mesh_view.draw(encoder, &mut self.factory, &self.camera, view, &mut self.editor_grid);
        for mut m in self.course.meshes() {
            self.mesh_view.draw(encoder, &mut self.factory, &self.camera, view, &mut m);
        }
        self.mesh_view.draw(encoder, &mut self.factory, &self.camera, view, &mut self.glider.mesh);
        self.line_view.draw(encoder, &self.camera, view);

    }

}


// Main -----------------------------------------------------------------------
pub fn main() {
    renderer::run::<Game, _>("Glider", 800, 600, 60, move |refs| {
        Game::new(refs)
    });
}

