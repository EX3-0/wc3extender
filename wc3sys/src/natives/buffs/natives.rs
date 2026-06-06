use core::ffi::c_void;
use std::ffi::CString;

use wc3::{alloc_executable, calc_rel32, read_bytes, write_bytes};

use crate::{addresses, engines, logging};

const APPLY_BUFF_FACTORY_WRAPPER_LEN: usize = 245;
const APPLY_BUFF_FACTORY_RAWCODE_PATCH_OFFSETS: [usize; 3] = [0x16, 0x4D, 0xA5];

const VT_CONFIG: usize = 0x32C;
const VT_BASE_BIND: usize = 0x328;

type RawUnit = usize;
type RawEffect = usize;

type UnitHandleToCUnitFn = unsafe extern "C" fn(handle: u32) -> RawUnit;

type UnitFindEffectFn = unsafe extern "thiscall" fn(
    unit: RawUnit,
    rawcode: u32,
    alias_match: i32,
    compare_stored_rawcode: i32,
    include_normal: i32,
    include_hidden: i32,
) -> RawEffect;

type CreateCbufEffectFn = unsafe extern "thiscall" fn(
    out_slot: *mut RawEffect,
    a2: i32,
    a3: i32,
    a4: i32,
) -> *mut RawEffect;

type AttachEffectToUnitFn = unsafe extern "thiscall" fn(unit: RawUnit, effect: RawEffect) -> i32;

type EffectConfigFn = unsafe extern "thiscall" fn(
    effect: RawEffect,
    config: *const u8,
) -> i32;

type EffectBaseBindFn = unsafe extern "thiscall" fn(
    effect: RawEffect,
    target: RawUnit,
    duration: *const f32,
    show_art: i32,
) -> i32;

unsafe fn vfunc<T>(object: usize, offset: usize) -> T {
    let vtable = unsafe { (object as *const usize).read_unaligned() };
    let target = unsafe { ((vtable + offset) as *const usize).read_unaligned() };
    unsafe { core::mem::transmute_copy::<usize, T>(&target) }
}

unsafe fn unit_handle_to_cunit(handle: u32) -> RawUnit {
    if handle == 0 {
        return 0;
    }

    let f: UnitHandleToCUnitFn = unsafe {
        core::mem::transmute(addresses::get().unit_handle_to_cunit)
    };

    unsafe { f(handle) }
}

unsafe fn find_effect_by_stored_rawcode(unit: RawUnit, rawcode: u32) -> RawEffect {
    let f: UnitFindEffectFn = unsafe {
        core::mem::transmute(addresses::get().unit_find_effect)
    };

    unsafe { f(unit, rawcode, 0, 1, 1, 1) }
}

unsafe fn create_effect_with_factory(create_addr: usize) -> RawEffect {
    let create: CreateCbufEffectFn = unsafe {
        core::mem::transmute(create_addr)
    };

    let mut slot: RawEffect = 0;
    unsafe {
        create(&mut slot as *mut RawEffect, 0, 0, 0);
    }

    let effect = slot;
    if effect == 0 {
        return 0;
    }

    unsafe {
        let refcount = (effect + 4) as *mut u32;
        let cur = refcount.read_unaligned();
        refcount.write_unaligned(cur.wrapping_sub(1));
        if cur == 1 {
            let dtor: unsafe extern "thiscall" fn(RawEffect) = vfunc(effect, 0);
            dtor(effect);
            return 0;
        }
    }

    effect
}

unsafe fn bind_apply_buff(
    effect: RawEffect,
    target: RawUnit,
    duration: f32,
    config: *const u8,
) -> bool {
    if effect == 0 || target == 0 || duration <= 0.0 {
        return false;
    }

    let cfg: EffectConfigFn = unsafe { vfunc(effect, VT_CONFIG) };
    let bind: EffectBaseBindFn = unsafe { vfunc(effect, VT_BASE_BIND) };

    unsafe {
        cfg(effect, config);
        bind(effect, target, &duration, 1);
    }

    true
}

unsafe fn create_effect_with_apply_buff_factory(rawcode: u32) -> RawEffect {
    let template_addr = addresses::get().create_bslo_effect;
    let template_len = APPLY_BUFF_FACTORY_WRAPPER_LEN;
    let patch_offsets = &APPLY_BUFF_FACTORY_RAWCODE_PATCH_OFFSETS;

    let Some(factory_addr) = (unsafe { alloc_executable(template_len) }) else {
        return 0;
    };

    let mut bytes = unsafe { read_bytes(template_addr, template_len) };

    for &off in patch_offsets {
        if off + 4 > template_len {
            return 0;
        }

        bytes[off..off + 4].copy_from_slice(&rawcode.to_le_bytes());
    }

    let mut i = 0usize;
    while i + 5 <= template_len {
        if bytes[i] == 0xE8 {
            let old_disp = i32::from_le_bytes([
                bytes[i + 1],
                bytes[i + 2],
                bytes[i + 3],
                bytes[i + 4],
            ]);

            let old_next = template_addr.wrapping_add(i + 5);
            let target = (old_next as isize).wrapping_add(old_disp as isize) as usize;
            let new_disp = calc_rel32(factory_addr.wrapping_add(i), target);

            bytes[i + 1..i + 5].copy_from_slice(&new_disp.to_le_bytes());
            i += 5;
        } else {
            i += 1;
        }
    }

    if unsafe { write_bytes(factory_addr, &bytes) }.is_err() {
        return 0;
    }

    unsafe { create_effect_with_factory(factory_addr) }
}

unsafe fn apply_buff_to_unit(target: RawUnit, rawcode: u32, duration: f32) -> bool {
    if target == 0 || rawcode == 0 || duration <= 0.0 {
        return false;
    }

    let mut config = [0u8; 32];
    config[0..4].copy_from_slice(&rawcode.to_le_bytes());

    let existing = unsafe { find_effect_by_stored_rawcode(target, rawcode) };
    if existing != 0 {
        return unsafe { bind_apply_buff(existing, target, duration, config.as_ptr()) };
    }

    let effect = unsafe { create_effect_with_apply_buff_factory(rawcode) };
    if effect == 0 {
        return false;
    }

    if !unsafe { bind_apply_buff(effect, target, duration, config.as_ptr()) } {
        return false;
    }

    let attach: AttachEffectToUnitFn = unsafe {
        core::mem::transmute(addresses::get().attach_effect_to_unit)
    };

    unsafe {
        attach(target, effect);
    }

    true
}

pub unsafe extern "C" fn apply_buff_native(
    target_handle: u32,
    rawcode: u32,
    duration_ptr: u32,
) -> u32 {
    if duration_ptr == 0 || rawcode == 0 {
        return 0;
    }

    let duration = unsafe { (duration_ptr as *const f32).read_unaligned() };
    let target = unsafe { unit_handle_to_cunit(target_handle) };

    if unsafe { apply_buff_to_unit(target, rawcode, duration) } {
        1
    } else {
        0
    }
}

pub fn register_custom_natives() {
    let natives = [
        ("ApplyBuff", "(Hunit;IR)B", apply_buff_native as *const c_void),
    ];

    for (name, sig, func) in natives {
        let c_name = CString::new(name).unwrap();
        let c_sig = CString::new(sig).unwrap();

        match engines::request_plugin_native(c_name, c_sig, func) {
            Ok(_) => logging::info(&format!("status: queued {} for registration", name)),
            Err(e) => logging::error(&format!("status: failed to queue {}: {}", name, e)),
        }
    }
}
