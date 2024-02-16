use std::path::Path;

use tobj::LoadOptions;

use crate::renderer::buffers::vertexbuffer::Vertex;

pub struct Model {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
}

/*
impl Model {
    pub fn from_obj(model_path: &Path) -> Self {
        let model_obj = 
            tobj::load_obj(model_path, &LoadOptions{
                single_index: true,
                ..Default::default()
            })
                .expect("Failed to load model object!");

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
                        1.0,
                    ],
                    color: [1.0, 1.0, 1.0, 1.0],
                    tex_coord: [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]],
                };
                vertices.push(vertex);
            }

            indices = mesh.indices.clone();
        }

        Self {
            vertices,
            indices
        }
    }
}
*/

impl Model {
    pub fn from_obj(model_path: &Path) -> Self {
        let model_obj = 
            tobj::load_obj(model_path, &LoadOptions{
                single_index: true,
                ..Default::default()
            })
                .expect("Failed to load model object!");

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

        Self {
            vertices,
            indices
        }
    }
}