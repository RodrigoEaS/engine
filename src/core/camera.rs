use cgmath::{Angle, Deg, Matrix4, Rad, SquareMatrix, Vector3};

#[derive(Clone, Copy)]
pub struct ProjectionViewObject {
    pub(crate) view: Matrix4<f32>,
    pub(crate) proj: Matrix4<f32>
}

pub struct Camera {
    pub(crate) position: Vector3<f32>, 
    pub(crate) rotation: Vector3<f32>,

    pub(crate) fovy: Rad<f32>, 
    pub(crate) aspect: f32, 
    pub(crate) near: f32, 
    pub(crate) far: f32
}

impl Camera {
    pub fn new(extent: (f32, f32)) -> Camera {
        Camera {
            position: Vector3 { x: 1.0, y: 1.0, z: 1.0 },
            rotation: Vector3 { x: 1.0, y: 1.0, z: 1.0 },

            fovy: Rad(45.0),
            aspect: extent.0 / extent.1,
            near: 0.1,
            far: 100.0,
        }
    }

    pub fn get_view(&self) -> Matrix4<f32> {
        let mut rotation_matrix = Matrix4::identity();
        let translate_matrix = Matrix4::from_translation(self.position);

        rotation_matrix += Matrix4::from_angle_x(Deg(self.rotation.x));
		rotation_matrix += Matrix4::from_angle_y(Deg(self.rotation.y));
		rotation_matrix += Matrix4::from_angle_z(Deg(self.rotation.z));

        rotation_matrix * translate_matrix
    }

    pub fn get_projection(&self) -> Matrix4<f32> {     
        //assert!(glm::abs(aspect - std::numeric_limits<float>::epsilon()) > 0.0f);
        
        let mut projection_matrix = Matrix4::identity();

        let tan_half_fovy = (self.fovy / 2.0).tan();
        projection_matrix[0][0] = 1.0 / (self.aspect * tan_half_fovy);
        projection_matrix[1][1] = 1.0 / (tan_half_fovy);
        projection_matrix[2][2] = self.far / (self.far - self.near);
        projection_matrix[2][3] = 1.0;
        projection_matrix[3][2] = -(self.far * self.near) / (self.far - self.near);


        //invert the y axis for vulkan
        //projection_matrix[1][1] = projection_matrix[1][1] * -1.0;

        projection_matrix
    }
}