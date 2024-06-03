#[derive(Clone, Copy)]
pub enum CameraStatus {
    Active,
    Sleep,
}

impl std::fmt::Display for CameraStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CameraStatus::Active => write!(f, "1"),
            CameraStatus::Sleep => write!(f, "0"),
        }
    }
}
