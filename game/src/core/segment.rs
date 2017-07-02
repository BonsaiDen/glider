// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::f32::consts::PI;


// External Dependencies ------------------------------------------------------
use cgmath::{Vector3, Quaternion, Matrix4, Deg, Euler, Transform};
use renderer::{Keyboard, Key};


// Internal Dependencies ------------------------------------------------------
use ::core::{Mesh, Bezier, Point, Loop, Row};
use ::render::LineView;


// 3D Course Segment Implementation -------------------------------------------
pub struct Segment {
    from: Point,
    to: Point,

    angle: f32,
    active_point: bool,
    typ: SegmentType,

    rows: Vec<Row>,
    mesh: Mesh
}

impl Segment {

    pub fn new(from: Point) -> Self {
        let mut segment = Self {
            from: from.clone(),
            to: from,

            angle: 0.0,
            active_point: false,
            typ: SegmentType::Straight,

            rows: Vec::new(),
            mesh: Mesh::from_raw(Vec::new(), Vec::new())
        };
        let origin = segment.from.pos;
        segment.set_to_straight(origin);
        segment.generate();
        segment
    }

    pub fn edit(&mut self, keyboard: &Keyboard) {

        if keyboard.was_pressed(Key::G) {
            self.active_point = !self.active_point;
        }

        let origin = if self.active_point {
            self.to.pos

        } else {
            self.from.pos
        };

        if keyboard.was_pressed(Key::Key1) {
            self.set_to_straight(origin);
            self.generate();
        }

        if keyboard.was_pressed(Key::Key2) {
            self.set_to_90_curve(origin);
            self.generate();
        }

        if keyboard.was_pressed(Key::Key3) {
            self.set_to_180_curve(origin);
            self.generate();
        }

        if keyboard.was_pressed(Key::Key4) {
            self.set_to_looping(origin);
            self.generate();
        }

        if keyboard.was_pressed(Key::U) {
            self.rotate(origin, -90.0);
            self.generate();
        }

        if keyboard.was_pressed(Key::O) {
            self.rotate(origin, 90.0);
            self.generate();
        }

        let shift = keyboard.is_pressed(Key::LShift);
        if keyboard.was_pressed(Key::I) {
            self.translate(Vector3::new(100.0, 0.0, 0.0), false);
            if shift {
                self.translate(Vector3::new(100.0, 0.0, 0.0), true);
            }
            self.generate();
        }

        if keyboard.was_pressed(Key::K) {
            self.translate(Vector3::new(-100.0, 0.0, 0.0), false);
            if shift {
                self.translate(Vector3::new(-100.0, 0.0, 0.0), true);
            }
            self.generate();
        }

        if keyboard.was_pressed(Key::J) {
            self.translate(Vector3::new(0.0, 0.0, -100.0), false);
            if shift {
                self.translate(Vector3::new(0.0, 0.0, -100.0), true);
            }
            self.generate();
        }

        if keyboard.was_pressed(Key::L) {
            self.translate(Vector3::new(0.0, 0.0, 100.0), false);
            if shift {
                self.translate(Vector3::new(0.0, 0.0, 100.0), true);
            }
            self.generate();
        }

    }

    pub fn start_point(&self) -> Vector3<f32> {
        self.from.pos
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn mesh_mut(&mut self) -> &mut Mesh {
        &mut self.mesh
    }

    pub fn debug(&mut self, lines: &mut LineView) {

        let (b, c, _, _) = self.control_points();
        lines.add(b.pos, b.pos + Vector3::new(0.0, 100.0, 0.0), [255.0, 128.0, 0.0, 1.0]);
        lines.add(c.pos, c.pos + Vector3::new(0.0, 100.0, 0.0), [0.0, 128.0, 255.0, 1.0]);

        if self.active_point {
            lines.add(self.to.pos,  self.to.pos + Vector3::new(0.0, 300.0, 0.0), [255.0, 255.0, 0.0, 1.0]);

        } else {
            lines.add(self.from.pos,  self.from.pos + Vector3::new(0.0, 300.0, 0.0), [255.0, 255.0, 0.0, 1.0]);
        }

        for row in &self.rows {
            let angle = 0.0f32;
            let o = (row.binormal * angle.cos() + row.normal * angle.sin()) * 50.0;
            lines.add(row.pos, row.pos + o, [0.0, 255.0, 0.0, 1.0]);
        }

    }

}


// Internal Interface ---------------------------------------------------------
impl Segment {

    fn set_to_straight(&mut self, origin: Vector3<f32>) {
        self.angle = 0.0;
        self.from.roll = 0.0;
        self.to.roll = 0.0;
        self.typ = SegmentType::Straight;

        if self.active_point {
            self.to.pos = origin;
            self.from.pos = origin - Vector3::new(500.0, 0.0, 0.0);

        } else {
            self.from.pos = origin;
            self.to.pos = origin + Vector3::new(500.0, 0.0, 0.0);
        }
    }

    fn set_to_180_curve(&mut self, origin: Vector3<f32>) {
        self.angle = 180.0;
        self.from.roll = 0.0;
        self.to.roll = 0.0;
        self.typ = SegmentType::Curve180;

        if self.active_point {
            self.to.pos = origin;
            self.from.pos = origin - Vector3::new(0.0, 0.0, 500.0);

        } else {
            self.from.pos = origin;
            self.to.pos = origin + Vector3::new(0.0, 0.0, 500.0);
        }
    }

    fn set_to_90_curve(&mut self, origin: Vector3<f32>) {
        self.angle = 180.0;
        self.from.roll = 0.0;
        self.to.roll = 0.0;
        self.typ = SegmentType::Curve90;

        if self.active_point {
            self.to.pos = origin;
            self.from.pos = origin - Vector3::new(300.0, 0.0, 300.0);

        } else {
            self.from.pos = origin;
            self.to.pos = origin + Vector3::new(300.0, 0.0, 300.0);
        }
    }

    fn set_to_looping(&mut self, origin: Vector3<f32>) {
        self.angle = 0.0;
        self.from.roll = 0.0;
        self.to.roll = 0.0;
        self.typ = SegmentType::Looping;

        if self.active_point {
            self.to.pos = origin;
            self.from.pos = origin - Vector3::new(500.0, 0.0, 0.0);

        } else {
            self.from.pos = origin;
            self.to.pos = origin + Vector3::new(500.0, 0.0, 0.0);
        }
    }

    fn rotate(&mut self, origin: Vector3<f32>, angle: f32) {
        self.angle = (self.angle + angle) % 360.0;

        //let angle = (PI / 180.0) * angle;
        let df = self.from.pos - origin;
        let dt = self.to.pos - origin;

        let r: Matrix4<f32> = Quaternion::from(Euler {
            x: Deg(0.0),
            y: Deg(-angle),
            z: Deg(0.0)

        }).into();

        self.from.pos = origin + r.transform_vector(df);
        self.to.pos = origin + r.transform_vector(dt);

    }

    fn translate(&mut self, offset: Vector3<f32>, invert: bool) {
        if self.active_point {
            if invert {
                self.from.pos += offset;

            } else {
                self.to.pos += offset;
            }

        } else {
            if invert {
                self.to.pos += offset;

            } else {
                self.from.pos += offset;
            }
        }
    }

    // TODO two sided shader?
    fn generate(&mut self) {

        let (rows, fa, ta) = match self.typ {
            SegmentType::Looping => {

                let dx = (self.from.pos.x - self.to.pos.x).abs();
                let dz = (self.from.pos.z - self.to.pos.z).abs();
                let height = if self.angle == 0.0 || self.angle == 180.0 {
                    dx

                } else {
                    dz
                };

                let looping = Loop::new(
                    self.from.clone(),
                    self.to.clone(),
                    height,
                    self.angle
                );
                (looping.generate_segments(50.0), -self.angle, -self.angle)

            },
            _ => {
                let (b, c, fa, ta) = self.control_points();
                let color = [1.0, 1.0, 0.0, 1.0];
                let a = self.from.clone();
                let d = self.to.clone();
                let bezier = Bezier::new(a, b, c, d);
                (bezier.generate_segments(50.0), fa, ta)
            }
        };


        let (v, i) = triangulate(&rows[..], 3, fa, ta);
        self.mesh = Mesh::from_raw(v, i);
        self.mesh.set_color([1.0, 1.0, 0.0, 1.0]);
        self.rows = rows;

    }

    fn control_points(&self) -> (Point, Point, f32, f32) {
        match self.typ {
            SegmentType::Curve180 => {
                let v = self.to.pos - self.from.pos;
                let u = Vector3::new(-v.z, 0.0, v.x);
                let s = 2.0 / 3.0;

                let b = self.from.pos - u * s;
                let c = self.to.pos - u * s;
                (
                    Point::new(b.x, b.y, b.z, self.from.width, self.from.roll),
                    Point::new(c.x, c.y, c.z, self.to.width, self.to.roll),
                    (self.angle + 180.0) % 360.0,
                    self.angle
                )
            },
            SegmentType::Curve90 => {
                let v = self.to.pos - self.from.pos;
                let (u, w) = if self.angle == 0.0 || self.angle == 180.0 {
                    (Vector3::new(v.x, 0.0, 0.0), Vector3::new(0.0, 0.0, -v.z))

                } else if self.angle == 90.0 || self.angle == 270.0 {

                    (Vector3::new(0.0, 0.0, v.z), Vector3::new(-v.x, 0.0, 0.0))
                } else {
                    (Vector3::new(0.0, 0.0, v.z), Vector3::new(-v.x, 0.0, 0.0))
                };

                let s = 0.55228;

                let b = self.from.pos + u * s;
                let c = self.to.pos + w * s;
                (
                    Point::new(b.x, b.y, b.z, self.from.width, self.from.roll),
                    Point::new(c.x, c.y, c.z, self.to.width, self.to.roll),
                    self.angle + 180.0,
                    (self.angle + 270.0) % 360.0
                )
            },
            SegmentType::Straight => {
                // TODO support adjusting controls points
                let dx = (self.from.pos.x - self.to.pos.x).abs();
                let dz = (self.from.pos.z - self.to.pos.z).abs();
                let d = dx.max(dz) * 1.33;
                (
                    self.from.rotate_around(self.angle, d * 0.5),
                    self.to.rotate_around(self.angle + 180.0, d * 0.5),
                    self.angle,
                    self.angle
                )
            },
            SegmentType::Looping => {
                (self.from.clone(), self.to.clone(), self.angle, self.angle)
            }
        }
    }

}


// Helpers --------------------------------------------------------------------
enum SegmentType {
    Straight,
    Curve90,
    Curve180,
    Looping
}

pub fn triangulate(
    rows: &[Row],
    cols: u32,
    fa: f32,
    ta: f32

) -> (Vec<Vector3<f32>>, Vec<u32>) {

    let mut vertices = Vec::with_capacity(rows.len() * cols as usize);
    let mut indices = Vec::new();

    let last = rows.len().saturating_sub(1);
    for (index, s) in rows.iter().enumerate() {

        let angle = PI * 0.5 + (PI / 180.0) * s.roll;

        // Correct yaw rotation for start and end rows
        let mut normal = s.normal;
        if index == 0 {
            let r = PI * 0.5 + (PI / 180.0) * fa;
            normal = Vector3::new(r.cos() * -1.0, 0.0, r.sin() * -1.0);

        } else if index == last {
            let r = PI * 0.5 + (PI / 180.0) * ta;
            normal = Vector3::new(r.cos() * -1.0, 0.0, r.sin() * -1.0);
        }

        let mut o = (s.binormal * angle.cos() + normal * angle.sin()) * s.width;
        let step = o * (2.0 / cols as f32);
        for _ in 0..cols + 1 {
            vertices.push((s.pos + o)) ;
            o -= step;
        }

    }

    let len = vertices.len() as u32;
    let l = len.saturating_sub(cols + 1);

    let mut i = 0;
    while i < l {
        for b in 0..cols  {
            indices.push( i + b + 0 );
            indices.push((i + b + cols + 1) % len);
            indices.push((i + b + 1) % len);

            indices.push((i + b + cols + 1) % len);
            indices.push((i + b + cols + 2) % len);
            indices.push((i + b + 1) % len);
        }
        i += cols + 1;

    }

    (vertices, indices)

}


