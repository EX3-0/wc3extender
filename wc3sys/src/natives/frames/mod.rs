pub mod natives;
pub mod structs;
pub mod hooks;

pub mod events;
pub mod trigger_events;
pub mod types;
pub mod offsets;
pub mod frame_registry;
pub mod ops;
pub mod frame_type;

pub use types::*;

pub fn init() -> crate::error::Result<()> {
    hooks::install()
}
