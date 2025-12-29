//! 3D interaction tools for user input

pub mod picking;
pub mod navigation;

pub use picking::{Picker, PickResult, Ray};
pub use navigation::{NavigationController, CameraMode, NavigationInput};

use crate::{Camera, Scene, Error, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_module() {
        // Basic test
    }
}
