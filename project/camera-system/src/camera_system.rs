use crate::camera::Camera;

pub struct CameraSystem {
    cameras: Vec<Camera>,
}

impl CameraSystem {
    pub fn new() -> Self {
        return CameraSystem { cameras: vec![] };
    }

    pub fn add_camera(mut self, camera: Camera) {
        self.cameras.push(camera)
    }

    pub fn new_incident() {}
}
