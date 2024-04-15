use cgmath::{Matrix4, Vector3, Vector4};

pub trait Transform {
    fn transform(&self) -> Matrix4<f32>;
}

pub struct EntityJoin{
    entities: Vec<Entity>
}

impl EntityJoin {
    pub fn new() -> Self {
        Self {
            entities: Vec::new()
        }
    }

    pub(crate) fn add(&mut self, entity: Entity) {
        self.entities.push(entity)
    }

    pub(crate) fn get_transforms(&self) -> Vec<Matrix4<f32>> {
        self.entities.iter().map(|x| -> Matrix4<f32> {
                x.transform()
            }
        ).collect()
    }
}

pub struct Entity {
    pub(crate) position: Vector3<f32>,
    pub(crate) scale: Vector3<f32>,
    pub(crate) rotation: Vector3<f32>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            position: Vector3 { x: 1.0, y: 1.0, z: 1.0 },
            scale: Vector3 { x: 1.0, y: 1.0, z: 1.0 },
            rotation: Vector3 { x: 1.0, y: 1.0, z: 1.0 },
        }
    }

    /*
    pub fn with_position(mut self, pos: Vector3<f32>) -> Self {
        self.position = pos;
        self
    }

    pub fn with_scale(mut self, scale: Vector3<f32>) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_rotation(mut self, rot: Vector3<f32>) -> Self {
        self.rotation = rot;
        self
    }
    */
}

impl Transform for Entity {
    // Matrix corrsponds to Translate * Ry * Rx * Rz * Scale
    // Rotations correspond to Tait-bryan angles of Y(1), X(2), Z(3)
    // https://en.wikipedia.org/wiki/Euler_angles#Rotation_matrix
    fn transform(&self) -> Matrix4<f32> {
        let c3 = self.rotation.z.cos();
        let s3 = self.rotation.z.sin();
        let c2 = self.rotation.x.cos();
        let s2 = self.rotation.x.sin();
        let c1 = self.rotation.y.cos();
        let s1 = self.rotation.y.sin();

        Matrix4 { 
            x: Vector4 {
                x: self.scale.x * (c1 * c3 + s1 * s2 * s3),
                y: self.scale.x * (c2 * s3),
                z: self.scale.x * (c1 * s2 * s3 - c3 * s1),
                w: 0.0,
            }, 
            y: Vector4 {
                x: self.scale.y * (c3 * s1 * s2 - c1 * s3),
                y: self.scale.y * (c2 * c3),
                z: self.scale.y * (c1 * c3 * s2 + s1 * s3),
                w: 0.0,
            }, 
            z: Vector4 {
                x: self.scale.z * (c2 * s1),
                y: self.scale.z * (-s2),
                z: self.scale.z * (c1 * c2),
                w: 0.0,
            }, 
            w: Vector4 {
                x: self.position.x,
                y: self.position.y,
                z: self.position.z,
                w: 1.0,
            }
        } 
    }
}