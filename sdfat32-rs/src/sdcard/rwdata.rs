use super::{
    cmd::SdCommand,
    SdCard,
    SdCardError,
};
use avr_hal_generic::port::PinOps;

const BLOCK_SIZE: usize = 512;
static mut BUFFER: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];

#[derive(PartialEq, Eq)]
pub(crate) enum DataMode {
    Idle,
    Read,
    Write,
}

pub(crate) struct RwState {
    pub(crate) mode: DataMode,
    pub(crate) sector: u32,
}

impl<CSPIN: PinOps> SdCard<CSPIN> {
    pub fn read_sector_as<T>(&mut self, sector: u32) -> Result<&'static T, SdCardError> {
        if self.rw_state.mode != DataMode::Read || self.rw_state.sector != sector {
            self.select();
            self.send_card_command(SdCommand::ReadBlock, sector)?;
            self.rw_state.mode = DataMode::Read;
            self.rw_state.sector = sector;
            unsafe {
                self.read_data(&mut BUFFER)?;
            }
            self.unselect();
        }

        unsafe { Ok(core::mem::transmute(BUFFER.as_ptr())) }
    }
}
