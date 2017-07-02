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


// External Dependencies ------------------------------------------------------
use cgmath::Matrix4;
use renderer::{ColorBuffer, DepthBuffer};


// Internal Dependencies ------------------------------------------------------
use ::core::{Camera, Mesh};


// 3D Mesh Rendering Implementation -------------------------------------------
#[derive(Debug)]
pub struct MeshView {
    pso: gfx::PipelineState<gfx_device_gl::Resources, mesh::Meta>,
    locals: gfx::handle::Buffer<gfx_device_gl::Resources, Locals>,
    out_color: ColorBuffer,
    out_depth: DepthBuffer
}

impl MeshView {

    pub fn new(
        factory: &mut gfx_device_gl::Factory,
        color: ColorBuffer,
        depth: DepthBuffer

    ) -> Self {
        Self {
            pso: MeshView::create_pipeline(factory, true).unwrap(),
            locals: factory.create_constant_buffer(1),
            out_color: color,
            out_depth: depth
        }
    }

    pub fn create_pipeline(factory: &mut gfx_device_gl::Factory, wireframe: bool) -> Result<gfx::PipelineState<gfx_device_gl::Resources, mesh::Meta>, Box<Error>> {

        let vertex = load_shader("mesh.vs")?;
        let fragment = load_shader("mesh.fs")?;

        let shader_program = factory.link_program(
            &vertex[..],
            &fragment[..]
        )?;

        let mut r = Rasterizer::new_fill();
        if wireframe {
            r.method = gfx::state::RasterMethod::Line(1);
        }
        r.samples = None;

        Ok(factory.create_pipeline_from_program(
            &shader_program,
            gfx::Primitive::TriangleList,
            r,
            mesh::new()

        ).unwrap())
    }

    pub fn reload(&mut self, factory: &mut gfx_device_gl::Factory, wireframe: bool) {
        match MeshView::create_pipeline(factory, wireframe) {
            Ok(pso) => self.pso = pso,
            Err(err) => println!("{:?}", err)
        }
    }

    pub fn resize(&mut self, screen: ((u32, u32), ColorBuffer, DepthBuffer)) {
        self.out_color = screen.1;
        self.out_depth = screen.2;
    }

    pub fn draw(
        &mut self,
        encoder: &mut gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
        factory: &mut gfx_device_gl::Factory,
        camera: &Camera,
        view: Matrix4<f32>,
        mesh: &mut Mesh
    ) {

        if !mesh.is_rendered() {
            mesh.render(factory)
        }

        let locals = Locals {
            model: mesh.transform.into(),
            view: view.into(),
            proj: camera.projection().into(),
        };

        encoder.update_buffer(&self.locals, &[locals], 0).unwrap();

        let data = mesh::Data {
            vbuf: mesh.buffer.as_ref().unwrap().clone(),
            locals: self.locals.clone(),
            blend_target: self.out_color.clone(),
            blend_ref: [1.0; 4],
            out_color: self.out_color.clone(),
            out_depth: self.out_depth.clone()
        };

        encoder.draw(mesh.slice.as_ref().unwrap(), &self.pso, &data);

    }

}


// Data -----------------------------------------------------------------------
gfx_defines!{
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        color: [f32; 4] = "a_Color",
    }

    constant Locals {
        model: [[f32; 4]; 4] = "u_Model",
        view: [[f32; 4]; 4] = "u_View",
        proj: [[f32; 4]; 4] = "u_Proj",
    }

    pipeline mesh {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        locals: gfx::ConstantBuffer<Locals> = "Transform",
        blend_target: gfx::BlendTarget<gfx::format::Srgba8> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
        blend_ref: gfx::BlendRef = (),
        out_color: gfx::RenderTarget<gfx::format::Srgba8> = "Target0",
        out_depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

pub use self::Vertex as MeshVertex;


fn load_shader(filename: &str) -> Result<Vec<u8>, io::Error> {

    let mut path = PathBuf::new();
    path.push("../assets/shaders/");
    path.push(filename);

    let mut file = File::open(&path)?;
    let mut code = Vec::new();
    file.read_to_end(&mut code)?;
    Ok(code)
}

