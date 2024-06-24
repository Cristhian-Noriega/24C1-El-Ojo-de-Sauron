use serde_derive::{Deserialize, Serialize};

/// Represents a position in 2D space
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Coordenate {
    pub x_coordinate: f64,
    pub y_coordinate: f64,
}
