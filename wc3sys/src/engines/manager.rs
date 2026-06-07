use core::ffi::c_void;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crate::error::Result;
use crate::logging;

use super::{CallbackContext, Engine, EngineContext};
use super::{handlers, lua, natives};

const CALLBACK_RESERVED_BASE: u32 = 0x80000000;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ManagerHandle;

impl ManagerHandle {
    pub(crate) fn mint_callback(self, engine_name: &'static str, opaque: u64) -> u32 {
        mint_callback(engine_name, opaque)
    }
}

#[derive(Debug, Clone)]
struct CallbackEntry {
    engine_name: &'static str,
    opaque: u64,
}

static ENGINES: Mutex<Vec<Arc<dyn Engine>>> = Mutex::new(Vec::new());
static CALLBACKS: OnceLock<Mutex<HashMap<u32, CallbackEntry>>> = OnceLock::new();
static NEXT_CALLBACK_ID: AtomicU32 = AtomicU32::new(CALLBACK_RESERVED_BASE);
static MAIN_CALLED: AtomicBool = AtomicBool::new(false);
fn handle() -> ManagerHandle {
    ManagerHandle
}

fn callbacks() -> &'static Mutex<HashMap<u32, CallbackEntry>> {
    CALLBACKS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn snapshot() -> Vec<Arc<dyn Engine>> {
    ENGINES.lock().unwrap().clone()
}

pub fn install(engine: Arc<dyn Engine>) -> Result<()> {
    let context = EngineContext::new(handle(), engine.name());
    engine.install(context)?;
    ENGINES.lock().unwrap().push(engine);
    Ok(())
}

pub fn mint_callback(engine_name: &'static str, opaque: u64) -> u32 {
    let id = NEXT_CALLBACK_ID.fetch_add(1, Ordering::Relaxed);
    callbacks().lock().unwrap().insert(
        id,
        CallbackEntry {
            engine_name,
            opaque,
        },
    );
    id
}

fn clear_callbacks() {
    callbacks().lock().unwrap().clear();
    NEXT_CALLBACK_ID.store(CALLBACK_RESERVED_BASE, Ordering::Relaxed);
}

fn resolve_callback(code_id: u32) -> Option<CallbackEntry> {
    if code_id < CALLBACK_RESERVED_BASE {
        return None;
    }
    callbacks().lock().unwrap().get(&code_id).cloned()
}

pub fn try_dispatch_callback_code(code_id: u32, context: CallbackContext) -> bool {
    if code_id < CALLBACK_RESERVED_BASE {
        return false;
    }

    let Some(entry) = resolve_callback(code_id) else {
        logging::warn(&format!("dispatch_callback_code: stale code_id 0x{code_id:x}"));
        return true;
    };

    for engine in snapshot() {
        if engine.name() == entry.engine_name {
            engine.dispatch_callback(entry.opaque, context);
            return true;
        }
    }

    logging::warn(&format!(
        "dispatch_callback_code: engine '{}' not found",
        entry.engine_name
    ));
    true
}


fn register_native_for_all(name: &str, signature: &str, func: *const c_void) {
    for engine in snapshot() {
        engine.register_native(name, signature, func);
    }
}

fn map_payload_for(engine: &Arc<dyn Engine>) -> Option<Vec<u8>> {
    let sig = engine.map_signature()?;
    let marker = crate::archives::read_file(sig.marker_path)?;
    logging::info(&format!(
        "engines: map signature matched for {} at {}",
        engine.name(),
        sig.marker_path
    ));

    if sig.payload_path == sig.marker_path {
        return Some(marker);
    }

    match crate::archives::read_file(sig.payload_path) {
        Some(payload) => Some(payload),
        None => {
            logging::warn(&format!(
                "engines: map payload missing for {} at {}",
                engine.name(),
                sig.payload_path
            ));
            None
        }
    }
}

fn config_all() {
    MAIN_CALLED.store(false, Ordering::Relaxed);
    clear_callbacks();
    crate::natives::frames::events::clear();

    let native_snapshot = natives::snapshot();

    for engine in snapshot() {
        let payload = map_payload_for(&engine);
        engine.set_map_payload(payload);
        engine.config();
        for rec in &native_snapshot {
            engine.register_native(&rec.name, &rec.signature, rec.func as *const c_void);
        }
    }
}

fn function_called_for_all(name: &str) {
    for engine in snapshot() {
        engine.function_called(name);
    }
}

pub(super) fn on_jass_native_registered(name: &str, signature: &str, func: *const c_void) {
    natives::observe_registered(name, signature, func);
    register_native_for_all(name, signature, func);
}

pub(super) fn on_jass_function_called(name: &str) {
    if name == "config" {
        config_all();
    }

    if name == "main" && MAIN_CALLED.swap(true, Ordering::Relaxed) {
        logging::info("engines: main skipped; already dispatched for this config");
        return;
    }

    function_called_for_all(name);
}

pub(super) fn on_jass_native_registration_phase() {
    natives::flush_pending();
}

pub fn init() -> Result<()> {
    natives::init();
    handlers::install()?;

    install(Arc::new(lua::LuaEngine::new()))?;

    logging::info("engines: initialized");
    Ok(())
}
