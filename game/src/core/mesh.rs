// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// External Dependencies ------------------------------------------------------
use gfx;
use gfx_device_gl;
use gfx::traits::FactoryExt;
use genmesh::{Vertices, Triangulate};
use genmesh::generators::{Plane, Cube, SharedVertex, IndexedPolygon};
use cgmath::{Matrix4, SquareMatrix, Vector3, InnerSpace, Zero};


// Internal Dependencies ------------------------------------------------------
use ::render::MeshVertex;


// 3D Mesh Implementation -----------------------------------------------------
pub struct Mesh {
    vectors: Vec<Vector3<f32>>,
    indices: Vec<u32>,
    triangles: Vec<(u32, u32, u32)>,
    color: [f32; 4],

    pub transform: Matrix4<f32>,
    pub buffer: Option<gfx::handle::Buffer<gfx_device_gl::Resources, MeshVertex>>,
    pub slice: Option<gfx::Slice<gfx_device_gl::Resources>>
}

impl Mesh {

    pub fn from_grid_plane(w: f32, h: f32, tx: usize, ty: usize) -> Self {

        let ws = (w / tx as f32) * (tx as f32 / 2.0);
        let hs = (h / ty as f32) * (ty as f32 / 2.0);

        let plane = Plane::subdivide(tx, ty);
        let vertex_data: Vec<Vector3<f32>> = plane.shared_vertex_iter()
            .map(|m| {
                let (x, y) = (m.pos[0], m.pos[1]);
                Vector3::new((x + 1.0) * ws, 0.0, (y + 1.0) * hs)
            })
            .collect();

        let index_data: Vec<u32> = plane.indexed_polygon_iter()
            .triangulate()
            .vertices()
            .map(|i| i as u32)
            .collect();

        Mesh::from_raw(vertex_data, index_data)

    }

    pub fn from_cube(w: f32, h: f32, d: f32) -> Self {

        let cube = Cube::new();
        let vertex_data: Vec<Vector3<f32>> = cube.shared_vertex_iter()
            .map(|m| {
                let (x, y, z) = (m.pos[0], m.pos[1], m.pos[2]);
                Vector3::new(x * w, z * h, y * d)
            })
            .collect();

        let index_data: Vec<u32> = cube.indexed_polygon_iter()
            .triangulate()
            .vertices()
            .map(|i| i as u32)
            .collect();

        Mesh::from_raw(vertex_data, index_data)

    }

    pub fn from_raw(vertices: Vec<Vector3<f32>>, indices: Vec<u32>) -> Self {
        let triangles = indices.chunks(3).map(|i| (i[0], i[1], i[2])).collect();
        Self {
            vectors: vertices,
            indices: indices,
            triangles: triangles,
            color: [1.0; 4],
            transform: Matrix4::identity(),
            buffer: None,
            slice: None
        }
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.buffer = None;
        self.slice = None;
        self.color = color;
    }

    pub fn triangles(&self) -> Vec<(Vector3<f32>, Vector3<f32>, Vector3<f32>)> {
        self.triangles.iter().map(|i| {
            (
                self.vectors[i.0 as usize],
                self.vectors[i.1 as usize],
                self.vectors[i.2 as usize]
            )
        }).collect()
    }

    pub fn intersect_ray(&self, ray: (Vector3<f32>, Vector3<f32>), tid: usize) -> Option<Intersection> {

        let indices = self.triangles[tid];
        let triangle = [
            &self.vectors[indices.0 as usize],
            &self.vectors[indices.1 as usize],
            &self.vectors[indices.2 as usize]
        ];

        let p = intersect_ray_triangle(ray, &triangle);
        if let Intersection::PointAndNormal(p, n) = p {
            Some(Intersection::PointAndNormal(p, n))

        } else {
            None
        }

    }

    pub fn is_rendered(&self) -> bool {
        self.buffer.is_some()
    }

    pub fn render(&mut self, factory: &mut gfx_device_gl::Factory) {

        println!("[Mesh] Rendering...");

        let vertices: Vec<MeshVertex> = self.vectors.iter().map(|v| {
            MeshVertex {
                pos: [v.x, v.y, v.z],
                color: self.color
            }

        }).collect();

        let (buf, slice) = factory.create_vertex_buffer_with_slice(&vertices, &self.indices[..]);
        self.buffer = Some(buf);
        self.slice = Some(slice);

    }

}


// Helpers --------------------------------------------------------------------
pub enum Intersection {
    Degenerate,
    Parallel,
    PointAndNormal(Vector3<f32>, Vector3<f32>),
    None
}

pub fn intersect_ray_triangle(r: (Vector3<f32>, Vector3<f32>), t: &[&Vector3<f32>; 3]) -> Intersection {

    // get triangle edge vectors and plane normal
    let u = t[1] - t[0];
    let v = t[2] - t[0];

    // cross product
    let n = u.cross(v);

    // triangle is degenerate
    if n.is_zero() {
        return Intersection::Degenerate;
    }

    // Ray Direction Vector
    let dir = r.1 - r.0;
    let w0 = r.0 - t[0];
    let a = -n.dot(w0);
    let b = n.dot(dir);

    // ray is parallel to triangle plane
    if b.abs() < 0.000001 {
        return Intersection::Parallel;
    }

    // get intersect point of ray with triangle plane
    let rr = a / b;
    if rr < 0.0 || rr > 1.0 {
        return Intersection::None; // Segment does not intersect with plane
    }

    // Intersection point of segment and plane
    let i = r.0 + rr * dir;

    // Is i inside the triangle?
    let uu = u.dot(u);
    let uv = u.dot(v);
    let vv = v.dot(v);
    let w = i - t[0];
    let wu = w.dot(u);
    let wv = w.dot(v);
    let d = uv * uv - uu * vv;

    // Get and test parametic coords
    let s = (uv * wv - vv * wu) / d;
    if s < 0.0 || s > 1.0 {
        return Intersection::None; // i is outside of triangle
    }

    let t = (uv * wu - uu * wv) / d;
    if t < 0.0 || (s + t) > 1.0 {
        return Intersection::None; // i is outside of triangle
    }

    Intersection::PointAndNormal(i, -n.normalize())

}

