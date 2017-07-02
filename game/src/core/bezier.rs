// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// External Dependencies ------------------------------------------------------
use cgmath::{Vector3, InnerSpace};


// 3D Bezier Implementation ---------------------------------------------------
#[derive(Debug)]
pub struct Bezier {
    // TODO use simple Vector3 for control points
    points: (Point, Point, Point, Point)
}

impl Bezier {

    pub fn new(a: Point, b: Point, c: Point, d: Point) -> Self {
        Self {
            points: (a, b, c, d)
        }
    }

    pub fn generate_segments(&self, step: f32) -> Vec<Row> {

        let mut segments = Vec::new();
        let (r1, r2) = (self.points.1.roll,  self.points.2.roll);
        let (w1, w2) = (self.points.1.width, self.points.2.width);

        let mut t = 0.0f32;
        loop {

            let py = self.point(t.min(1.0), true);
            let p = self.point(t.min(1.0), false);
            let p2 = self.point((t + 0.002), false);

            let deriv = self.derivative(t);
            let len = deriv.magnitude();
            let ta = (p2 - p).normalize();
            let mut b = ta.cross(p2 + p).normalize();
            b.y = b.y.abs();

            let n = b.cross(ta).normalize();
            segments.push(Row {
                pos: py,
                binormal: b,
                normal: n,
                width: lerp(w1, w2, t),
                roll: lerp(r1, r2, t)
            });

            if t >= 1.0 {
                break;
            }

            t += step / len;

        }

        segments

    }

    fn point(&self, t: f32, with_y: bool) -> Vector3<f32> {
		let dt = 1.0 - t;
		let dt2 = dt * dt;
		let t2 = t * t;
        let ref p = self.points;

		let mut v = (p.0.pos * dt2 * dt) + (p.1.pos * 3.0 * dt2 * t) + (p.2.pos * 3.0 * dt * t2) + (p.3.pos * t2 * t);
        if !with_y {
            v.y = 0.0;
        }
        v
    }

    fn derivative(&self, t: f32) -> Vector3<f32> {
		let dt = 1.0 - t;
		let dt2 = dt * dt;
		let t2 = t * t;
        let ref p = self.points;
        ((p.1.pos - p.0.pos) * dt2 * 3.0) + ((p.2.pos - p.1.pos) * dt * t * 6.0) + ((p.3.pos - p.2.pos) * t2 * 3.0)
    }

}


// Helpers --------------------------------------------------------------------
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

#[derive(Debug, Clone)]
pub struct Point {
    pub pos: Vector3<f32>,
    pub width: f32,
    pub roll: f32
}

impl Point {

    pub fn new(x: f32, y: f32, mut z: f32, width: f32, roll: f32) -> Self {
        if z == 0.0 {
            z = 0.00001;
        }

        Point {
            pos: Vector3::new(x, y, z),
            width: width,
            roll: roll
        }
    }

    pub fn rotate_around(&self, angle: f32, distance: f32) -> Point {
        let angle = ::std::f32::consts::PI / 180.0 * angle;
        Point::new(
            self.pos.x + angle.cos() * distance,
            self.pos.y,
            self.pos.z + angle.sin() * distance,
            self.width,
            self.roll
        )
    }

}

#[derive(Debug)]
pub struct Row {
    pub pos: Vector3<f32>,
    pub binormal: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub width: f32,
    pub roll: f32
}

