mod snapshot;
mod verify;

pub use snapshot::TagSnapshot;
pub use verify::tags;
pub use verify::verify_preserved;
pub use verify::Violation;
pub use verify::ViolationKind;

pub mod prelude {
    pub use super::*;
}
