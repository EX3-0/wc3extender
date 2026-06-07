mod callbacks;
mod dev_script;
mod invoke;
mod marshal;
mod native_bindings;
mod runtime;
mod sig;

use core::ffi::c_void;
use std::sync::Arc;

use mlua::Function;

use super::{CallbackContext, Engine, EngineContext, EngineMapSignature};
use crate::logging;

use runtime::LuaRuntime;

pub struct LuaEngine {
    runtime: Arc<LuaRuntime>,
}

impl LuaEngine {
    pub fn new() -> Self {
        Self {
            runtime: LuaRuntime::new_shared(),
        }
    }
}

impl Engine for LuaEngine {
    fn name(&self) -> &'static str { "lua" }

    fn install(&self, context: EngineContext) -> crate::error::Result<()> {
        self.runtime.install(context)?;
        Ok(())
    }

    fn map_signature(&self) -> Option<EngineMapSignature> {
        Some(EngineMapSignature {
            marker_path: "war3map.lua",
            payload_path: "war3map.lua",
        })
    }

    fn set_map_payload(&self, payload: Option<Vec<u8>>) {
        let source = payload.and_then(|bytes| match String::from_utf8(bytes) {
            Ok(source) => Some(source),
            Err(e) => {
                logging::warn(&format!("[lua] map payload is not utf-8: {e}"));
                None
            }
        });
        self.runtime.set_map_script(source);
    }

    fn config(&self) {
        if let Err(e) = self.runtime.rebuild() {
            logging::error(&format!("LuaEngine config: {e}"));
        }
    }

    fn function_called(&self, name: &str) {
        let Some(lua) = self.runtime.lua() else { return };
        let func: mlua::Result<Function> = lua.globals().get(name);
        let Ok(func) = func else { return };

        if let Err(e) = func.call::<()>(()) {
            logging::warn(&format!("[lua] {name}: {e}"));
        }
    }

    fn register_native(&self, name: &str, signature: &str, func: *const c_void) {
        if let Err(e) = native_bindings::register_native(self.runtime.clone(), name, signature, func) {
            logging::warn(&format!("[lua] register_native {name}: {e}"));
        }
    }

    fn dispatch_callback(&self, opaque: u64, context: CallbackContext) {

        let _instance_guard = match context {

            CallbackContext::InvokeCodeById { jass_instance_index } => {

                Some(super::handlers::enter_jass_instance_index(jass_instance_index))

            }

        };



        if let Err(e) = self.runtime.dispatch_callback(opaque) {

            logging::warn(&format!("[lua] dispatch_callback: {e}"));

        }

    }

}
