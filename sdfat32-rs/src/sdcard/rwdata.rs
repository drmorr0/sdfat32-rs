use super::{
    cmd::SdCommand,
    SdCard,
    SdCardError,
};
use avr_hal_generic::port::PinOps;
use core::marker::PhantomData;

#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum DataMode {
    Idle,
    Locked,
    Read,
    Write,
}

pub const BLOCK_SIZE: usize = 512;
static mut BUFFER: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
static mut BUFFER_MODE: DataMode = DataMode::Idle;
static mut SECTOR_IN_BUFFER: u32 = 0;

pub struct Block<T> {
    old_buffer_mode: DataMode,
    object: PhantomData<T>,
}

impl<T> Block<T> {
    pub(crate) fn new() -> Block<T> {
        let old_buffer_mode: DataMode;
        unsafe {
            old_buffer_mode = BUFFER_MODE;
            BUFFER_MODE = DataMode::Locked;
        }
        Block {
            old_buffer_mode,
            object: PhantomData,
        }
    }

    pub(crate) fn get(&self) -> &'static T {
        unsafe { core::mem::transmute(BUFFER.as_ptr()) }
    }
}

impl<T> Drop for Block<T> {
    fn drop(&mut self) {
        unsafe {
            BUFFER_MODE = self.old_buffer_mode;
        }
    }
}

fn read_required(sector: u32) -> bool {
    unsafe { BUFFER_MODE != DataMode::Read || SECTOR_IN_BUFFER != sector }
}

fn is_locked() -> bool {
    unsafe { BUFFER_MODE == DataMode::Locked }
}

impl<CSPIN: PinOps> SdCard<CSPIN> {
    pub fn read_sector_as<T>(&mut self, sector: u32) -> Result<Block<T>, SdCardError> {
        if is_locked() {
            return Err(SdCardError::DataBufferLocked);
        } else if read_required(sector) {
            self.select();
            self.send_card_command(SdCommand::ReadBlock, sector)?;
            unsafe {
                BUFFER_MODE = DataMode::Read;
                SECTOR_IN_BUFFER = sector;
                self.read_data(&mut BUFFER)?;
            }
            self.unselect();
        }

        Ok(Block::new())
    }
}
