use std::{mem::size_of, path::Path, rc::Rc};

use ash::vk;
use memoffset::offset_of;
use tobj::LoadOptions;

use crate::{core::device::GraphicDevice, renderer::{buffer::Buffer, commandpool::CommandPool}};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl Vertex {
    pub fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Self, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Self, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Self, tex_coord) as u32,
            },
        ]
    }
}

pub struct Mesh {
    device: Rc<GraphicDevice>,

    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,

    pub(crate) index_count: u32,
}

impl Mesh {
    pub fn from_obj( 
        device: Rc<GraphicDevice>, 
        command_pool: &CommandPool, 
        model_path: &Path
    ) -> Self {
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
        
        //VERTEX BUFFER
        let vertex_size = (size_of::<Vertex>() * vertices.len()) as u64;

        let vertex_staging_buffer = Buffer::staging(device.clone(), vertex_size);
        vertex_staging_buffer.map(&vertices, vertex_size);

        let vertex_buffer = Buffer::vertex(device.clone(), vertex_size);
        vertex_buffer.copy(
            &vertex_staging_buffer,
            command_pool, 
            vertex_size
        );

        vertex_staging_buffer.destroy();

        //INDEX BUFFER
        let index_size = (size_of::<u32>() * indices.len()) as u64;

        let index_staging_buffer = Buffer::staging(device.clone(), index_size);
        index_staging_buffer.map(&indices, index_size);

        let index_buffer = Buffer::index(device.clone(), index_size);
        index_buffer.copy(
            &index_staging_buffer,
            command_pool, 
            index_size
        );
        
        index_staging_buffer.destroy();
        
        Self {
            device,

            vertex_buffer,
            index_buffer,

            index_count: indices.len() as u32,
        }
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

    pub(crate) fn draw(&self, command_buffer: vk::CommandBuffer, count: u32) {
        unsafe {
            self.device.logical.cmd_draw_indexed(
                command_buffer, 
                self.index_count, 
                count, 
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
