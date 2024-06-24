/// The different statuses a camera can have.
#[derive(Clone, Copy, Debug, PartialEq)]
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

impl CameraStatus {
    pub fn to_str(&self) -> String {
        match self {
            CameraStatus::Active => "Active".to_string(),
            CameraStatus::Sleep => "Inactive".to_string(),
        }
    }
}
