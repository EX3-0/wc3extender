pub mod frames;

pub mod buffs;

pub fn init() -> crate::error::Result<()> {
    frames::init()?;
    buffs::init();
    Ok(())
}
