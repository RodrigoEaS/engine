use std::{path::Path, rc::Rc};

use ash::vk;
use tobj::LoadOptions;

use crate::{core::device::GraphicDevice, renderer::buffers::{indexbuffer::IndexBuffer, vertexbuffer::{Vertex, VertexBuffer}}};

pub struct Mesh {
    device: Rc<GraphicDevice>,

    pub(crate) vertex_buffer: VertexBuffer,
    pub(crate) index_buffer: IndexBuffer,

    pub(crate) index_count: u32,
}

impl Mesh {
    pub fn from_obj( 
        device: Rc<GraphicDevice>, 
        command_pool: &vk::CommandPool, 
        model_path: &Path
    ) -> Rc<Self> {
        let model_obj = tobj::load_obj(
            model_path, &LoadOptions{
                single_index: true,
                ..Default::default()
            }
        ).expect("Failed to load model object!");

        let mut vertices = vec![];
        let mut indices = vec![];

        let (models, _) = model_obj;
        for m in models.iter() {
            let mesh = &m.mesh;

            if mesh.texcoords.len() == 0 {
                panic!("Missing texture coordinate for the model.")
            }

            let total_vertices_count = mesh.positions.len() / 3;
            for i in 0..total_vertices_count {
                let vertex = Vertex {
                    pos: [
                        mesh.positions[i * 3],
                        mesh.positions[i * 3 + 1],
                        mesh.positions[i * 3 + 2],
                    ],
                    color: [1.0, 1.0, 1.0],
                    tex_coord: [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]],
                };
                vertices.push(vertex);
            }

            indices = mesh.indices.clone();
        }

        let vertex_buffer = VertexBuffer::new(
            device.clone(), command_pool, &vertices
        );

        let index_buffer = IndexBuffer::new(
            device.clone(), command_pool, &indices
        );

        Rc::new(Self {
            device,

            vertex_buffer,
            index_buffer,

            index_count: indices.len() as u32,
        }) 
    }

    pub(crate) fn bind(&self, command_buffer: vk::CommandBuffer) {
        let vertex_buffers = [self.vertex_buffer.buffer];
        let offsets = [0_u64];

        unsafe {
            self.device.logical.cmd_bind_vertex_buffers(
                command_buffer, 
                0, 
                &vertex_buffers, 
                &offsets
            );
            self.device.logical.cmd_bind_index_buffer(
                command_buffer,
                self.index_buffer.buffer,
                0,
                vk::IndexType::UINT32,
            );
        }
    }

    pub(crate) fn draw(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.logical.cmd_draw_indexed(
                command_buffer, 
                self.index_count, 
                1, 
                0, 
                0, 
                0
            );
        }
    }

    pub(crate) fn destroy(&self) {
        self.vertex_buffer.destroy();
        self.index_buffer.destroy();
    }
}
