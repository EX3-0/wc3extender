use core::ffi::c_void;

use crate::{addresses, archives, engines, jass, logging, natives, plugins, ui};

pub unsafe extern "system" fn worker_thread(_param: *mut c_void) -> u32 {
    let _ = logging::init();

    if let Err(e) = addresses::init_from_process() {
        logging::error(&format!("address init failed: {e}"));
        return 1;
    }

    if let Err(e) = ui::init() {
        logging::error(&format!("ui events init failed: {e}"));
        return 1;
    }

    if let Err(e) = jass::custom_triggers::install() {
        logging::error_value("custom triggers hooks init failed", &e);
        return 1;
    }

    if let Err(e) = natives::init() {
        logging::error_value("natives init failed", &e);
        return 1;
    }

    if let Err(e) = engines::init() {
        logging::error_value("engines init failed", &e);
        return 1;
    }

    if let Err(e) = archives::init() {
        logging::error_value("archives init failed", &e);
        return 1;
    }

    if let Err(e) = plugins::init() {
        logging::error(&format!("plugins init failed: {e}"));
        return 1;
    }

    logging::info("wc3sys boot complete");
    0
}
