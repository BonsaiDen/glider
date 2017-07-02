// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::f32::consts::PI;


// External Dependencies ------------------------------------------------------
use cgmath::Vector3;


// Internal Dependencies ------------------------------------------------------
use ::core::{Point, Row};


// 3D Bezier Loop Implementation ----------------------------------------------
#[derive(Debug)]
pub struct Loop {
    points: (Point, Point),
    pub angle: f32,
    width: f32,
    radius: f32
}

impl Loop {

    pub fn new(a: Point, b: Point, height: f32, rotation: f32) -> Self {
        Self {
            angle: 90.0 - rotation,
            width: a.width + b.width,
            points: (a, b),
            radius: height
        }
    }

    pub fn generate_segments(&self, step: f32) -> Vec<Row> {

        let mut segments = Vec::new();
        let length = self.radius * PI * 2.0;

        let angle = self.angle * (PI / 180.0);
        let offset_angle = (self.angle - 90.0) * (PI / 180.0);

        let ox = offset_angle.sin() * self.width;
        let oz = offset_angle.cos() * self.width;

        let mut t = 0.0f32;
        while t < length * 2.0 {

            let l = (t.min(length) / self.radius) * 0.5;
            let u = t.min(length) / self.radius - PI * 0.5;
            let w = lerp(self.points.1.width, self.points.0.width, l / PI);
            let x = self.points.0.pos.x + angle.sin() * u.cos() * self.radius + lerp(0.0, ox, l / PI);
            let y = self.points.0.pos.y + u.sin() * self.radius + self.radius;
            let z = self.points.0.pos.z + angle.cos() * u.cos() * self.radius + lerp(0.0, oz, l / PI);

            let n = Vector3::new(angle.cos() * -1.0, 0.0, angle.sin() * -1.0);

            // TODO calculate bi-normal outwards
            let b = -Vector3::new(angle.sin() * u.cos(), u.sin(), angle.cos() * u.cos());

            segments.push(Row {
                pos: Vector3::new(x, y, z),
                binormal: b,
                normal: n,
                width: w,
                roll: lerp(self.points.1.roll, self.points.0.roll, l / PI)
            });

            if t >= length {
                break;
            }

            t += step;

        }

        segments

    }

}


// Helpers --------------------------------------------------------------------
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

