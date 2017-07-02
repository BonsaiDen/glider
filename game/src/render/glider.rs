// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// External Dependencies ------------------------------------------------------
use gfx;
use gfx::traits::FactoryExt;
use gfx::state::Rasterizer;
use gfx_device_gl;
use std::fs::File;
use std::path::PathBuf;
use std::io::{self, Read};
use std::error::Error;

use cgmath::{Matrix4, SquareMatrix, Vector3};
use genmesh::{Vertices, Triangulate};
use genmesh::generators::{Cube, SharedVertex, IndexedPolygon};


// External Dependencies ------------------------------------------------------
use ::core::{Camera, Glider};
use renderer::{ColorBuffer, DepthBuffer};


// 3D Glider Rendering Implementation -----------------------------------------
#[derive(Debug)]
pub struct GliderView {
    pso: gfx::PipelineState<gfx_device_gl::Resources, glider::Meta>,
    data: glider::Data<gfx_device_gl::Resources>,
    slice: gfx::Slice<gfx_device_gl::Resources>
}

impl GliderView {

    pub fn new(
        factory: &mut gfx_device_gl::Factory,
        color: ColorBuffer,
        depth: DepthBuffer

    ) -> Self {

        let cube = Cube::new();
        let vertex_data: Vec<Vertex> = cube.shared_vertex_iter()
            .map(|m| {
                let (x, y, z) = (m.pos[0], m.pos[1], m.pos[2]);
                Vertex {
                    pos: [x * 5.0, z * 5.0, y * 5.0],
                    color: [1.0, 1.0, 0.0]
                }
            })
            .collect();

        let index_data: Vec<u32> = cube.indexed_polygon_iter()
            .triangulate()
            .vertices()
            .map(|i| i as u32)
            .collect();

        let (buf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, &index_data[..]);
        Self {
            pso: GliderView::create_pipeline(factory, false).unwrap(),
            data: glider::Data {
                buf: buf,
                transform: factory.create_constant_buffer(1),
                model: Matrix4::identity().into(),
                view: Matrix4::identity().into(),
                proj: Matrix4::identity().into(),
                out_color: color,
                out_depth: depth
            },
            slice: slice
        }
    }

    pub fn create_pipeline(factory: &mut gfx_device_gl::Factory, _: bool) -> Result<gfx::PipelineState<gfx_device_gl::Resources, glider::Meta>, Box<Error>> {

        let vertex = load_shader("glider.vs")?;
        let fragment = load_shader("glider.fs")?;

        let shader_program = factory.link_program(
            &vertex[..],
            &fragment[..]
        )?;

        let mut r = Rasterizer::new_fill();
        //if wireframe {
            r.method = gfx::state::RasterMethod::Line(1);
        //}
        r.samples = None;

        Ok(factory.create_pipeline_from_program(
            &shader_program,
            gfx::Primitive::TriangleList,
            r,
            glider::new()

        ).unwrap())
    }

    pub fn reload(&mut self, factory: &mut gfx_device_gl::Factory, wireframe: bool) {
        match GliderView::create_pipeline(factory, wireframe) {
            Ok(pso) => self.pso = pso,
            Err(err) => println!("{:?}", err)
        }
    }

    pub fn resize(&mut self, screen: ((u32, u32), ColorBuffer, DepthBuffer)) {
        self.data.out_color = screen.1;
        self.data.out_depth = screen.2;
    }

    pub fn draw(
        &mut self,
        encoder: &mut gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
        camera: &Camera,
        view: Matrix4<f32>,
        glider: &Glider
    ) {

        self.data.view = view.into();

        let transform = Transform {
            model: glider.transform().into(),
            view: self.data.view,
            proj: camera.projection().into(),
        };

        encoder.update_buffer(&self.data.transform, &[transform], 0).unwrap();
        encoder.draw(&self.slice, &self.pso, &self.data);

    }

}


// Data -----------------------------------------------------------------------
gfx_defines!{
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    constant Transform {
        model: [[f32; 4]; 4] = "u_Model",
        view: [[f32; 4]; 4] = "u_View",
        proj: [[f32; 4]; 4] = "u_Proj",
    }

    pipeline glider {
        buf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::ConstantBuffer<Transform> = "Transform",
        model: gfx::Global<[[f32; 4]; 4]> = "u_Model",
        view: gfx::Global<[[f32; 4]; 4]> = "u_View",
        proj: gfx::Global<[[f32; 4]; 4]> = "u_Proj",
        out_color: gfx::RenderTarget<gfx::format::Srgba8> = "Target0",
        out_depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}


fn load_shader(filename: &str) -> Result<Vec<u8>, io::Error> {

    let mut path = PathBuf::new();
    path.push("../assets/shaders/");
    path.push(filename);

    let mut file = File::open(&path)?;
    let mut code = Vec::new();
    file.read_to_end(&mut code)?;
    Ok(code)
}


