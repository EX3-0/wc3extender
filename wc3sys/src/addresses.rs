use std::sync::OnceLock;

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleA;

pub const IDA_BASE: usize = 0x400000;

macro_rules! game_addresses {
    ($($name:ident: $static_addr:literal,)+) => {
        #[derive(Debug, Clone, Copy)]
        pub struct GameAddrs {
            pub base: usize,
            $(pub $name: usize,)+
        }

        impl GameAddrs {
            fn from_base(base: usize) -> Self {
                Self {
                    base,
                    $($name: rebase(base, $static_addr),)+
                }
            }
        }
    };
}

game_addresses! {
    register_native: 0x8CCC80,
    make_jass_string: 0x474CE0,
    string_to_handle: 0x489CD0,
	jass_vm_global: 0x113B6F4,
    jass_string_to_cstr: 0x469010,
    jass_get_subsystem: 0x45DF90,
    jass_string_handle_to_arg: 0x8D3FE0,
    jass_string_handle_from_cstr: 0x8D7230,
    jass_instance_from_index: 0x8CA900,
    jass_string_arg_from_handle_by_index: 0x8CA970,
    jass_string_handle_from_cstr_by_index: 0x8CBE40,
    invoke_code_by_id: 0x8CC230,
    run_function: 0x8CC1C0,
    register_natives: 0x49B3C0,
    create_trigger: 0x48ECF0,
    is_trigger_enabled: 0x499910,
    get_triggering_trigger: 0x497120,
    get_trigger_event_id: 0x496DB0,
    get_trigger_player: 0x496E00,
    trigger_evaluate: 0x4AD210,
    trigger_execute: 0x4AD230,
    player: 0x4A4240,
	open_archive_file: 0x439740,
    object_data_table: 0x113D410,
    object_hash: 0x95C860,
    object_data_find: 0x5CFD50,
    object_data_create: 0x5CC370,
    object_selection_scale_get: 0x5C5440,
    selection_circle_radius_get: 0x6C27A0,
    unit_handle_to_cunit: 0x483940,
    unit_find_effect: 0x6D6570,
    create_bpse_effect: 0xC00DF0,
    attach_effect_to_unit: 0x6D4880,
    apply_bslo_slow: 0xBF7F80,
    create_bslo_effect: 0xBF8090,
    apply_beer_roots: 0xC48FB0,
    create_beer_effect: 0xC490B0,
    create_bams_effect: 0xBAC6F0,
    create_bam2_effect: 0xBAC7F0,
    apply_bams_antimagic_shell: 0xBAC510,
    effect_registry_root: 0x115E9EC,
    effect_registry_find: 0x429B90,
    effect_descriptor_build: 0x7F2650,
    effect_materialize: 0x8A0660,
    rawcode_compatible: 0x7F55F0,
    // Frame API (Ported from W3CE Frame.cs)
    frame_setup_game_ui: 0x5E73F0,
    world_frame_set_cursor_mode: 0x616660,
    frame_setup_leaderboard: 0x5F3CE0,
    frame_setup_multiboard: 0x5F3F90,
    frame_create_world: 0x608C10,
    frame_create_leaderboard_item: 0x61F820,
    frame_create_unit_bar: 0x6218D0,
    frame_update_leaderboard_item: 0x657830,
    frame_create_multiboard_row: 0x65DA10,
    frame_update_multiboard_row: 0x660750,
    frame_sprite_clip: 0x7E65A0,
    frame_fd_file_read: 0x849C60,
    frame_simple_top_destructor: 0x889CA0,
    frame_def_create_frame: 0x8958B0,
    frame_def_create_simple_frame: 0x895930,
    frame_layer_find_under_cursor: 0x10E8EE8,
    frame_original_simple_top_ptr: 0x1161C4C,
    c_frame_registry_get_entry: 0x8711D0,
    c_simple_frame_registry_get_entry_a: 0x8915C0,
    c_simple_frame_registry_get_entry_b: 0x891700,
    c_simple_frame_registry_get_entry_c: 0x891470,
    c_backdrop_set_texture: 0x87C340,
    c_simple_texture_set_texture: 0x887C20,
    c_simple_status_bar_set_texture: 0x889620,
    c_game_ui_get_or_create: 0x5EC5E0,
    cursor_frame_get_or_create: 0x8744B0,
    c_layout_frame_set_all_points: 0x873750,
    c_layout_frame_clear_all_points: 0x872AA0,
    c_layout_frame_update: 0x8735F0,
    c_layout_frame_set_width: 0x873B40,
    c_layout_frame_set_height: 0x873830,
    c_layout_frame_set_point_abs: 0x873950,
    c_layout_frame_set_point: 0x873A10,
    c_simple_font_string_set_text: 0x887A40,
    c_text_frame_set_text: 0x87E380,
    c_text_area_set_text: 0x885C00,
    c_text_area_add_text: 0x885110,
    c_edit_box_set_text: 0x881C70,
    c_observer_dispatch_event: 0x83EDD0,
    c_observer_register_event: 0x83F470,
    c_simple_frame_register_event: 0xC74E80,
    c_frame_destroy: 0x876760,
    c_layer_set_alpha: 0x86F540,
    c_layer_set_owner: 0x86FE00,
    c_layer_set_tooltip: 0x86FF30,
    c_simple_frame_set_level: 0x600C60,
    c_simple_frame_set_parent: 0x889060,
    c_simple_frame_set_alpha: 0x888D90,
    c_simple_frame_set_layout_scale: 0x888FA0,
    c_simple_font_string_set_layout_scale: 0x8876D0,
    c_simple_glue_frame_set_layout_scale: 0x88C720,
    c_sprite_frame_set_layout_scale: 0x8786C0,
    c_layout_frame_cage_mouse: 0x872610,
    c_control_dispatch_click: 0x878CC0,
    c_control_enable: 0x52A620,
    c_control_check_state: 0x878C80,
    c_simple_button_set_enable: 0x874B80,
    c_text_frame_set_horizontal_justification: 0x87E190,
    c_text_frame_set_vertical_justification: 0x87E570,
    c_text_frame_update_control: 0x87E590,
    c_edit_box_set_focus: 0x881A60,
    c_simple_region_set_vertex_color: 0x886360,
    c_text_frame_set_text_color: 0x87E4A0,
    c_frame_set_font: 0x86F990,
    c_edit_box_set_font: 0x881B50,
    c_edit_box_set_text_size_limit: 0x881E50,
    c_simple_message_frame_set_font: 0x88C210,
    c_simple_font_string_set_font: 0x887470,
    c_slider_set_current_value: 0x87F090,
    c_simple_status_bar_set_value: 0x8897C0,
    c_simple_status_bar_set_min_max_value: 0x889770,
    c_status_bar_set_art: 0x893450,
    c_status_bar_set_value: 0x893550,
    c_status_bar_set_min_max_value: 0x8934E0,
    c_model_frame_add_model: 0x884560,
    c_sprite_frame_set_art: 0x8785E0,
    c_sprite_frame_get_sprite: 0x878190,
    c_sprite_uber_set_animation: 0x966C10,



    c_layer_show: 0x870020,
    c_layer_hide: 0x86D8C0,
    c_layer_find_under_cursor: 0x86CCF0,
    c_layer_active_layer: 0x1161AD0,
    c_layer_find_under_cursor_arg: 0x10E8EE8,

    string_hash_node_table: 0x1161CE0,
    frame_hash_node_table: 0x1161D38,
    toc_unk_global: 0x10E91A4,
    string_hash_node_grow: 0x84AAA0, // 8694432u
    base_frame_hash_node_grow: 0x84A410, // 8692752u


}


static GAME_ADDRS: OnceLock<GameAddrs> = OnceLock::new();

#[inline]
pub fn rebase(dynamic_base: usize, static_addr: usize) -> usize {
    static_addr - IDA_BASE + dynamic_base
}

pub fn init_from_process() -> Result<(), &'static str> {
    let dynamic_base = unsafe { GetModuleHandleA(core::ptr::null()) } as usize;
    if dynamic_base == 0 {
        return Err("GetModuleHandleA failed");
    }

    GAME_ADDRS
        .set(GameAddrs::from_base(dynamic_base))
        .map_err(|_| "GAME_ADDRS already set")
}

pub fn get() -> &'static GameAddrs {
    GAME_ADDRS.get().expect("GameAddrs not initialized")
}
