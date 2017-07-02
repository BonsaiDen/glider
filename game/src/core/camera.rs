// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::ops::Mul;


// External Dependencies ------------------------------------------------------
use cgmath::{self, Rotation3};
use cgmath::{Deg, Euler, Quaternion, Vector3, Matrix4};
use renderer::{Keyboard, Key};


// 3D Camera Implementation ---------------------------------------------------
pub struct Camera {
    fov: f32,
    pub rotation: Quaternion<f32>,
    pub position: Matrix4<f32>,
    projection: Matrix4<f32>
}

impl Camera {

    pub fn new(width: u32, height: u32, fov: f32) -> Self {
        let aspect_ratio = width as f32 / height as f32;
        Self {
            fov: fov,
            rotation: Quaternion::from(Euler {
                x: Deg(35.0),
                y: Deg(0.0),
                z: Deg(0.0),
            }),
            position: Matrix4::from_translation(Vector3::new(-100.0, -300.0, -600.0)),
            projection: cgmath::perspective(Deg(fov), aspect_ratio, 0.01, 15000.0)
        }
    }

    pub fn update(&mut self, keyboard: &Keyboard) {

        // Pitch down
        if keyboard.is_pressed(Key::W) {
            self.pitch(1.5);
        }

        // Pitch up
        if keyboard.is_pressed(Key::S) {
            self.pitch(-1.5);
        }

        // Yaw Left
        if keyboard.is_pressed(Key::A) {
            self.yaw(-2.5);
        }

        // Yaw Right
        if keyboard.is_pressed(Key::D) {
            self.yaw(2.5);
        }

        // Up
        if keyboard.is_pressed(Key::Q) {
            self.vertical(-7.5);
        }

        // Down
        if keyboard.is_pressed(Key::E) {
            self.vertical(7.5);
        }

        // Move Forward
        if keyboard.is_pressed(Key::Space) {
            self.forward(17.5);
        }

        // Move Backward
        if keyboard.is_pressed(Key::Backspace) {
            self.forward(-17.5);
        }

    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let aspect_ratio = width as f32 / height as f32;
        self.projection = cgmath::perspective(Deg(self.fov), aspect_ratio, 0.01, 15000.0);
    }

    pub fn view(&self) -> Matrix4<f32> {
        self.rotation().mul(self.position)
    }

    pub fn rotation(&self) -> Matrix4<f32> {
        self.rotation.into()
    }

    pub fn projection(&self) -> Matrix4<f32> {
        self.projection
    }

    fn pitch(&mut self, s: f32) {
        // TODO always apply rotation around X / Z axis
        self.rotation = Quaternion::from_angle_x(Deg(s)).mul(self.rotation);
    }

    fn yaw(&mut self, s: f32) {
        // TODO always apply rotation around Y axis
        self.rotation = self.rotation.mul(Quaternion::from_angle_y(Deg(s)));
        //self.rotation = Quaternion::from_angle_y(Deg(s)).mul(self.rotation);
    }

    //fn roll(&mut self, s: f32) {
    //    // TODO remove
    //    self.rotation = Quaternion::from_angle_z(Deg(s)).mul(self.rotation);
    //}

    fn forward(&mut self, s: f32) {
        let m: Matrix4<f32> = self.rotation.into();
        let step = Matrix4::from_translation(Vector3::new(0.0, 0.0, 1.0));
        let d = m.mul(step);
        self.position.w[0] += d.x[2] * s;
        self.position.w[1] += d.y[2] * s;
        self.position.w[2] += d.z[2] * s;
    }

    fn vertical(&mut self, s: f32) {
        self.position.w[1] += s;
    }

}

