// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// External Dependencies ------------------------------------------------------
use gfx;
use gfx::Factory;
use gfx::traits::FactoryExt;
use gfx::state::Rasterizer;
use gfx_device_gl;
use std::fs::File;
use std::path::PathBuf;
use std::io::{self, Read};
use std::error::Error;

// External Dependencies ------------------------------------------------------
use cgmath::{Matrix4, SquareMatrix, Vector3};
use renderer::{ColorBuffer, DepthBuffer};


// Internal Dependencies ------------------------------------------------------
use ::core::Camera;


// 3D Lines Rendering Implementation -----------------------------------------
#[derive(Debug)]
pub struct LineView {
    vertices: Vec<Vertex>,
    pso: gfx::PipelineState<gfx_device_gl::Resources, line::Meta>,
    data: line::Data<gfx_device_gl::Resources>,
    slice: gfx::Slice<gfx_device_gl::Resources>,
    dirty: bool,
    lines: usize,
    max_lines: usize
}

impl LineView {

    pub fn new(
        factory: &mut gfx_device_gl::Factory,
        color: ColorBuffer,
        depth: DepthBuffer,
        max_lines: usize

    ) -> Self {

        let mut vertices = Vec::with_capacity(max_lines);
        for _ in 0..max_lines {
            vertices.push(Vertex { pos: [0.0, 0.0, 0.0], color: [0.0, 0.0, 0.0, 1.0] });
            vertices.push(Vertex { pos: [0.0, 0.0, 0.0], color: [0.0, 0.0, 0.0, 1.0] });
        }

        let vertex_count = vertices.len();
        let vertex_buffer = factory.create_buffer::<Vertex>(
            vertex_count * 2,
            gfx::buffer::Role::Vertex,
            gfx::memory::Usage::Dynamic,
            gfx::Bind::empty()

        ).expect("QuadView: Could not create `vertex_buffer`");

        Self {
            vertices: vertices,
            pso: LineView::create_pipeline(factory, false).unwrap(),
            data: line::Data {
                buf: vertex_buffer,
                transform: factory.create_constant_buffer(1),
                model: Matrix4::identity().into(),
                view: Matrix4::identity().into(),
                proj: Matrix4::identity().into(),
                out_color: color,
                out_depth: depth
            },
            slice: gfx::Slice {
                instances: None,
                start: 0,
                end: vertex_count as u32,
                buffer: gfx::IndexBuffer::Auto,
                base_vertex: 0
            },
            dirty: true,
            lines: 0,
            max_lines: max_lines
        }
    }

    pub fn add(&mut self, from: Vector3<f32>, to: Vector3<f32>, color: [f32; 4]) {
        if self.lines < self.max_lines {
            let color = gamma_srgb_to_linear(color);
            self.vertices[self.lines * 2].pos = from.into();
            self.vertices[self.lines * 2].color = color;
            self.vertices[self.lines * 2 + 1].pos = to.into();
            self.vertices[self.lines * 2 + 1].color = color;
            self.lines += 1;
            self.dirty = true;
        }
    }

    pub fn reload(&mut self, factory: &mut gfx_device_gl::Factory, wireframe: bool) {
        match LineView::create_pipeline(factory, wireframe) {
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
        view: Matrix4<f32>
    ) {

        self.data.view = view.into();

        let transform = Transform {
            model: self.data.model,
            view: self.data.view,
            proj: camera.projection().into(),
        };

        if self.dirty {
            self.dirty = false;
            encoder.update_buffer(&self.data.buf, &self.vertices, 0).ok();
        }

        self.slice.end = (self.lines as u32) * 2;
        self.lines = 0;

        encoder.update_buffer(&self.data.transform, &[transform], 0).unwrap();
        encoder.draw(&self.slice, &self.pso, &self.data);

    }

    fn create_pipeline(factory: &mut gfx_device_gl::Factory, _: bool) -> Result<gfx::PipelineState<gfx_device_gl::Resources, line::Meta>, Box<Error>> {

        let vertex = load_shader("lines.vs")?;
        let fragment = load_shader("lines.fs")?;

        let shader_program = factory.link_program(
            &vertex[..],
            &fragment[..]
        )?;

        let mut r = Rasterizer::new_fill();
        r.method = gfx::state::RasterMethod::Line(2);
        r.samples = None;

        Ok(factory.create_pipeline_from_program(
            &shader_program,
            gfx::Primitive::LineList,
            r,
            line::new()

        ).unwrap())
    }

}


// Helpers --------------------------------------------------------------------
#[inline(always)]
fn component_srgb_to_linear(f: f32) -> f32 {
    if f <= 0.04045 {
        f / 12.92
    } else {
        ((f + 0.055) / 1.055).powf(2.4)
    }
}

fn gamma_srgb_to_linear(c: [f32; 4]) -> [f32; 4] {
    [
        component_srgb_to_linear(c[0] / 255.0),
        component_srgb_to_linear(c[1] / 255.0),
        component_srgb_to_linear(c[2] / 255.0),
        c[3]
    ]
}


// Data -----------------------------------------------------------------------
gfx_defines!{
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        color: [f32; 4] = "a_Color",
    }

    constant Transform {
        model: [[f32; 4]; 4] = "u_Model",
        view: [[f32; 4]; 4] = "u_View",
        proj: [[f32; 4]; 4] = "u_Proj",
    }

    pipeline line {
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

