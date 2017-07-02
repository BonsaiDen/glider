// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::collections::HashMap;


// External Dependencies ------------------------------------------------------
use cgmath::Vector3;
use renderer::{Keyboard, Key};


// Internal Dependencies ------------------------------------------------------
use ::core::{Mesh, Point, Segment, Intersection};
use ::render::LineView;



// 3D Course Implementation ---------------------------------------------------
pub struct Course {
    segments: Vec<Segment>,
    active_segment: usize,
    tree: Tree
}

impl Course {

    pub fn new() -> Self {

        // TODO handle segment indicies better
        let mut tree = Tree::new(250.0);
        let c = Segment::new(Point::new(0.0, 0.0, 0.0, 200.0, 0.0), 0.0);
        tree.insert(&c, 0);
        let segments = vec![c];
        Self {
            segments: segments,
            active_segment: 0,
            tree: tree
        }

    }

    pub fn start_point(&self) -> Vector3<f32> {
        self.segments[0].start_point()
    }

    pub fn meshes(&mut self) -> Vec<&mut Mesh> {
        self.segments.iter_mut().map(|s| s.mesh_mut()).collect()
    }

    pub fn intersect_ray(&self, ray: (Vector3<f32>, Vector3<f32>)) -> Intersection {
        self.tree.intersect_ray(ray, &self.segments[..])
    }

    pub fn edit(&mut self, keyboard: &Keyboard) {

        // TODO add new segment at start or end
        if self.segments.len() > self.active_segment {
            self.segments[self.active_segment].edit(keyboard);
        }

        /*

        // TODO move behind modes
        if keyboard.was_pressed(Key::Z) {
            if self.active_point_end {
                segment.to.roll = (segment.to.roll + 90.0) % 360.0;

            } else {
                segment.from.roll = (segment.from.roll + 90.0) % 360.0;
            }
            segment.refresh();
        }

        if keyboard.was_pressed(Key::H) {
            if self.active_point_end {
                segment.to.roll = (segment.to.roll - 90.0) % 360.0;

            } else {
                segment.from.roll = (segment.from.roll - 90.0) % 360.0;
            }
            segment.refresh();
        }

        if keyboard.was_pressed(Key::N) {
            if self.active_point_end {
                segment.to.width = segment.to.width + 50.0;

            } else {
                segment.from.width = segment.from.width + 50.0;
            }
            segment.refresh();
        }

        if keyboard.was_pressed(Key::M) {
            if self.active_point_end {
                segment.to.width = segment.to.width - 50.0;

            } else {
                segment.from.width = segment.from.width - 50.0;
            }
            segment.refresh();
        }
        */
    }

    pub fn debug(&mut self, lines: &mut LineView) {
        if self.segments.len() > self.active_segment {
            self.segments[self.active_segment].debug(lines);
        }
    }

}

struct Tree {
    // Maps grid cells to (segment, triangle) index combinations
    cells: HashMap<(i32, i32, i32), Vec<(usize, usize)>>,
    size: f32
}

impl Tree {

    pub fn new(size: f32) -> Self {
        Self {
            cells: HashMap::new(),
            size: size
        }
    }

    pub fn intersect_ray(&self, ray: (Vector3<f32>, Vector3<f32>), segments: &[Segment]) -> Intersection {

        let ix = (ray.0.x.min(ray.1.x) / self.size).floor() as i32;
        let iy = (ray.0.y.min(ray.1.y) / self.size).floor() as i32;
        let iz = (ray.0.z.min(ray.1.z) / self.size).floor() as i32;

        let mx = (ray.0.x.max(ray.1.x) / self.size).ceil() as i32;
        let my = (ray.0.y.max(ray.1.y) / self.size).ceil() as i32;
        let mz = (ray.0.z.max(ray.1.z) / self.size).ceil() as i32;

        for x in ix..mx + 1 {
            for y in iy..my + 1 {
                for z in iz..mz + 1 {
                    if let Some(pairs) = self.cells.get(&(x, y, z)) {
                        for &(sid, tid) in pairs {
                            if let Some(t) = segments[sid].mesh().intersect_ray(ray, tid) {
                                return t;
                            }
                        }
                    }
                }
            }
        }

        Intersection::None

    }

    pub fn insert(&mut self, s: &Segment, id: usize) {

        let triangles = s.mesh().triangles();
        for (i, t) in triangles.into_iter().enumerate() {

            let ix = (t.0.x.min(t.1.x).min(t.2.x) / self.size).floor() as i32;
            let iy = (t.0.y.min(t.1.y).min(t.2.y) / self.size).floor() as i32;
            let iz = (t.0.z.min(t.1.z).min(t.2.z) / self.size).floor() as i32;

            let mx = (t.0.x.max(t.1.x).max(t.2.x) / self.size).ceil() as i32;
            let my = (t.0.y.max(t.1.y).max(t.2.y) / self.size).ceil() as i32;
            let mz = (t.0.z.max(t.1.z).max(t.2.z) / self.size).ceil() as i32;

            for x in ix..mx + 1 {
                for y in iy..my + 1 {
                    for z in iz..mz + 1 {
                        let mut cell = self.cells.entry((x, y, z)).or_insert_with(Vec::new);
                        cell.push((id, i));
                    }
                }
            }

        }

    }

    pub fn remove(&mut self, s: &Segment) {
        // TODO go through all buckets
        // TODO and remove all entries with the given segment number
    }

}

