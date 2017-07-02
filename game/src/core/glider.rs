// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// External Dependencies ------------------------------------------------------
use renderer::{Keyboard, Key};
use cgmath::{Matrix4, Point3, Deg, Euler, Vector3, InnerSpace, Quaternion, Transform, Rotation};


// Internal Dependencies ------------------------------------------------------
use ::core::{Course, Intersection, Mesh};
use ::render::LineView;


// 3D Glider Logic Implementation ---------------------------------------------
pub struct Glider {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    airborne: bool,
    hover_height: f32,
    max_speed: f32,
    max_gravity: f32,
    fall: f32,
    accel: f32,
    brake: f32,
    turn: f32,
    smooth_y: f32,
    speed: f32,
    gravity: f32,
    yaw: f32,
    pub mesh: Mesh
}

impl Glider {

    pub fn new() -> Self {
        Self {
            position: Vector3::new(25.0, 0.0, 25.0),
            rotation: Quaternion::from(Euler {
                x: Deg(0.0),
                y: Deg(0.0),
                z: Deg(0.0)
            }),
            airborne: true,
            hover_height: 15.0,
            max_gravity: 6.0,
            max_speed: 2.0 * 60.0 * 0.5,
            accel: 0.025 * 60.0,
            fall: 2.0,
            brake: 0.075 * 60.0,
            turn: 1.5 * 60.0,
            smooth_y: 0.0,
            speed: 0.0,
            gravity: 0.0,
            yaw: 0.0,
            mesh: Mesh::from_cube(7.0 * 0.5, 4.0 * 0.5, 5.0 * 0.5)
        }
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.gravity = 0.0;
        self.speed = 0.0;
        self.position = position;
    }

    pub fn update(&mut self, dt: f32, course: &Course, lines: &mut LineView, keyboard: &Keyboard) {

        // Acceleration
        if keyboard.is_pressed(Key::W) && !self.airborne {
            self.speed += if self.speed >= self.max_speed {
                0.0

            } else {
                self.accel * dt
            };

        // Deceleration
        } else {
            self.speed = (self.speed - self.brake * dt).max(0.0);
        }

        // Gravity
        if self.airborne {
            self.gravity += if self.gravity >= self.max_gravity {
                0.0

            } else {
                self.fall * dt
            };

        } else {
            self.gravity = 0.0;
        }

        // Turning
        self.yaw = 0.0;
        if keyboard.is_pressed(Key::A) {
            self.yaw = (self.turn / (self.speed * 0.125).max(1.0)).min(self.turn) * dt;
            self.speed *= 0.998;
        }

        if keyboard.is_pressed(Key::D) {
            self.yaw = -(self.turn / (self.speed * 0.125).max(1.0)).min(self.turn) * dt;
            self.speed *= 0.998;
        }

        let m: Matrix4<f32> = self.rotation.into();
        let prev_up = m.transform_vector(Vector3::new(0.0, 1.0, 0.0)).normalize();
        let ahead = m.transform_vector(Vector3::new(40.0, 0.0, 0.0));
        let back = m.transform_vector(Vector3::new(20.0, 0.0, 0.0));

        // Perform intersection tests
        let ar = (
            self.position + ahead + prev_up * 20.0,
            self.position + ahead - prev_up * 50.0
        );
        lines.add(ar.0, ar.1, [128.0, 0.0, 255.0, 1.0]);

        let r = (
            self.position + prev_up * 30.0,
            self.position - prev_up * 30.0
        );
        lines.add(r.0, r.1, [255.0, 0.0, 255.0, 1.0]);

        let br = (
            self.position - back + prev_up * 20.0,
            self.position - back - prev_up * 50.0
        );
        lines.add(br.0, br.1, [128.0, 0.0, 255.0, 1.0]);

        let an = if let Intersection::PointAndNormal(_, n) = course.intersect_ray(ar) {
            Some(n)

        } else {
            None
        };

        let bn = if let Intersection::PointAndNormal(_, n) = course.intersect_ray(br) {
            Some(n)

        } else {
            None
        };

        if let Intersection::PointAndNormal(p, mut n) = course.intersect_ray(r) {

            if an.is_some() && bn.is_some() {
                n = (an.unwrap() + bn.unwrap() + n) / 3.0;
                //n = an.unwrap().lerp(bn.unwrap(), 0.5);
            }

            let distance = (p - self.position).magnitude();

            // Debug Normal display
            lines.add(p, p + n * 25.0, [0.0, 128.0, 128.0, 1.0]);

            // Calculate new up vector
            let desired_up = prev_up.lerp(n, 0.065 * 60.0 * dt);
            let tilt: Quaternion<f32> = Quaternion::between_vectors(prev_up, desired_up);
            self.rotation = tilt * self.rotation;

            // Smoothly adjust height
            self.smooth_y = lerp(self.smooth_y, self.hover_height - distance, 0.20 * 60.0 * dt).max(-distance).min(5.0);
            self.position += prev_up * self.smooth_y;
            self.airborne = false;

        } else {
            // TODO
            let n = Vector3::new(0.0, 1.0, 0.0);
            let desired_up = prev_up.lerp(n, 0.1 * 60.0 * dt);
            let tilt: Quaternion<f32> = Quaternion::between_vectors(prev_up, desired_up);
            self.rotation = tilt * self.rotation;
            self.position -= n * self.gravity;
            self.airborne = true;
        }

        self.rotation = self.rotation * Quaternion::from(Euler {
            x: Deg(0.0),
            y: Deg(self.yaw),
            z: Deg(0.0)
        });

        let m: Matrix4<f32> = self.rotation.into();
        let forward = m.transform_vector(Vector3::new(1.0, 0.0, 0.0)).normalize();
        self.position += forward * self.speed;

        self.mesh.transform = self.transform();

    }

    pub fn transform(&self) -> Matrix4<f32> {
        use std::ops::Mul;
        let r: Matrix4<f32> = self.rotation.into();
        let offset = Matrix4::from_translation(Vector3::new(0.0, -10.0, 0.0));
        Matrix4::from_translation(self.position).mul(r).mul(offset)
    }

    pub fn camera_view(&self) -> Matrix4<f32> {
        let t = self.transform();
        let c = Vector3::new(t.w[0], t.w[1], t.w[2]);
        let target = t.transform_vector(Vector3::new(0.0, 15.0, 0.0));
        let offset = t.transform_vector(Vector3::new(-37.0 - self.speed * 0.35, 15.0, -50.0 / (self.speed + 1.0)));
        let p = c + offset;
        let t = c + target;

        let m: Matrix4<f32> = self.rotation.into();
        let up = m.transform_vector(Vector3::new(0.0, 1.0, 0.0)).normalize();

        Matrix4::look_at(Point3::new(p.x, p.y, p.z), Point3::new(t.x, t.y, t.z), up)
    }

    pub fn debug(&self, lines: &mut LineView) {

        let t = self.transform();
        let p = Vector3::new(t.w[0], t.w[1], t.w[2]);
        let o = t.transform_vector(Vector3::new( 0.0,  0.0,  0.0)) + p;
        let x = t.transform_vector(Vector3::new(10.0,  0.0,  0.0)) + p;
        let y = t.transform_vector(Vector3::new( 0.0, 10.0,  0.0)) + p;
        let z = t.transform_vector(Vector3::new( 0.0,  0.0, 10.0)) + p;

        // X-Axis
        lines.add(o, x, [255.0, 0.0, 0.0, 1.0]);

        // Y-Axis
        lines.add(o, y, [0.0, 255.0, 0.0, 1.0]);

        // Z-Axis
        lines.add(o, z, [0.0, 0.0, 255.0, 1.0]);

    }

}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

