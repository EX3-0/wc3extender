pub mod natives;

pub fn init() {
    natives::register_custom_natives();
}
