use std::sync::{Arc, Mutex};

use mlua::{Lua, Value, Variadic};

use crate::engines::EngineContext;
use crate::logging;

use super::{callbacks::LuaCallbacks, dev_script};

pub struct LuaRuntime {
    lua: Mutex<Option<Lua>>,
    callbacks: LuaCallbacks,
    context: Mutex<Option<EngineContext>>,
    map_script: Mutex<Option<String>>,
}

impl LuaRuntime {
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self {
            lua: Mutex::new(None),
            callbacks: LuaCallbacks::new(),
            context: Mutex::new(None),
            map_script: Mutex::new(None),
        })
    }

    pub fn install(&self, context: EngineContext) -> Result<(), String> {
        let mut slot = self.context.lock().unwrap();
        if slot.is_some() {
            return Err("LuaRuntime already installed".to_string());
        }
        *slot = Some(context);
        Ok(())
    }

    pub fn set_map_script(&self, source: Option<String>) {
        *self.map_script.lock().unwrap() = source;
    }

    pub fn rebuild(self: &Arc<Self>) -> Result<(), String> {
        self.callbacks.clear();
        *self.lua.lock().unwrap() = None;

        let lua = Lua::new();
        self.install_print(&lua)?;
        lua.load(r#"print("LuaEngine ready")"#)
            .exec()
            .map_err(|e| e.to_string())?;
        let map_script = self.map_script.lock().unwrap().clone();
        let source = map_script.as_deref().unwrap_or(dev_script::MAIN);
        lua.load(source)
            .exec()
            .map_err(|e| e.to_string())?;

        *self.lua.lock().unwrap() = Some(lua);
        Ok(())
    }

    pub fn lua(&self) -> Option<Lua> {
        self.lua.lock().unwrap().as_ref().cloned()
    }

    pub fn callbacks(&self) -> &LuaCallbacks {
        &self.callbacks
    }

    pub fn mint_callback(&self, opaque: u64) -> Result<u32, String> {
        let context = self.context
            .lock()
            .unwrap()
            .ok_or_else(|| "LuaRuntime not installed".to_string())?;
        Ok(context.mint_callback(opaque))
    }

    pub fn dispatch_callback(&self, opaque: u64) -> Result<(), String> {
        let Some(lua) = self.lua() else {
            return Err("no lua state".to_string());
        };

        let func = self.callbacks.get(&lua, opaque)?;
        func.call::<()>(()).map_err(|e| e.to_string())
    }

    fn install_print(&self, lua: &Lua) -> Result<(), String> {
        let print = lua
            .create_function(|_, vals: Variadic<Value>| {
                let parts: Vec<String> = vals.iter().map(lua_value_to_debug_string).collect();
                logging::info(&format!("[lua] {}", parts.join(" ")));
                Ok(())
            })
            .map_err(|e| e.to_string())?;

        lua.globals().set("print", print).map_err(|e| e.to_string())
    }
}

fn lua_value_to_debug_string(v: &Value) -> String {
    match v {
        Value::Nil => "nil".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_string_lossy().to_string(),
        Value::Function(_) => "function".to_string(),
        Value::Table(_) => "table".to_string(),
        Value::Thread(_) => "thread".to_string(),
        Value::UserData(_) => "userdata".to_string(),
        Value::LightUserData(_) => "lightuserdata".to_string(),
        Value::Error(e) => format!("error: {e}"),
        Value::Other(_) => "other".to_string(),
    }
}
