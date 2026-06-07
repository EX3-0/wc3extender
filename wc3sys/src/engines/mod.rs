pub mod debug;

pub mod exports;

mod handlers;

pub mod lua;

mod manager;

pub mod natives;

use core::ffi::c_void;
use std::ffi::CString;

use crate::logging;

pub use handlers::{best_jass_instance_index, current_jass_instance_index};

#[derive(Debug, Clone, Copy)]
pub enum CallbackContext {
    InvokeCodeById { jass_instance_index: u32 },
}

#[derive(Debug, Clone, Copy)]
pub struct EngineMapSignature {
    pub marker_path: &'static str,
    pub payload_path: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct EngineContext {
    manager: manager::ManagerHandle,
    engine_name: &'static str,
}

impl EngineContext {
    pub(crate) fn new(manager: manager::ManagerHandle, engine_name: &'static str) -> Self {
        Self { manager, engine_name }
    }

    pub fn mint_callback(&self, opaque: u64) -> u32 {
        self.manager.mint_callback(self.engine_name, opaque)
    }
}

pub trait Engine: Send + Sync {
    fn name(&self) -> &'static str;

    fn install(&self, _context: EngineContext) -> crate::error::Result<()> {
        Ok(())
    }

    fn map_signature(&self) -> Option<EngineMapSignature> {
        None
    }

    fn set_map_payload(&self, _payload: Option<Vec<u8>>) {}

    fn config(&self);
    fn function_called(&self, name: &str);
    fn register_native(&self, name: &str, signature: &str, func: *const c_void);

    fn dispatch_callback(&self, _opaque: u64, _context: CallbackContext) {
        logging::warn(&format!("{}: dispatch_callback unsupported", self.name()));
    }
}

#[allow(dead_code)]
pub fn install(engine: std::sync::Arc<dyn Engine>) -> crate::error::Result<()> {
    manager::install(engine)
}

pub fn mint_callback(engine_name: &'static str, opaque: u64) -> u32 {
    manager::mint_callback(engine_name, opaque)
}

pub fn request_plugin_native(
    name: CString,
    signature: CString,
    func: *const c_void,
) -> Result<(), String> {
    natives::request_plugin_native(name, signature, func)
}

pub fn try_dispatch_callback_code(code_id: u32, context: CallbackContext) -> bool {
    manager::try_dispatch_callback_code(code_id, context)
}

pub fn init() -> crate::error::Result<()> {

    manager::init()

}
