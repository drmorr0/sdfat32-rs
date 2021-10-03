use super::{
    cmd::SdCommand,
    constants::*,
    SdCard,
    SdCardError,
};
use avr_hal_generic::port::PinOps;
use core::marker::PhantomData;


pub(crate) const BUFFER_COUNT: usize = 2;
pub(crate) const FS_BUFFER: usize = 0;
pub(crate) const DATA_BUFFER: usize = 1;
static mut BUFFER: [u8; BLOCK_SIZE * BUFFER_COUNT] = [0; BLOCK_SIZE * BUFFER_COUNT];
static mut BUFFER_MODE: [DataMode; BUFFER_COUNT] = [DataMode::Idle; 2];
static mut SECTOR_IN_BUFFER: [u32; BUFFER_COUNT] = [0; 2];

pub(crate) struct Block<T> {
    buffer_index: usize,
    old_buffer_mode: DataMode,
    object: PhantomData<T>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum DataMode {
    Idle,
    Locked,
    Read,
    Write,
}


impl<T> Block<T> {
    pub(crate) fn new(buffer_index: usize) -> Block<T> {
        let old_buffer_mode: DataMode;
        unsafe {
            old_buffer_mode = BUFFER_MODE[buffer_index];
            BUFFER_MODE[buffer_index] = DataMode::Locked;
        }
        Block { buffer_index, old_buffer_mode, object: PhantomData }
    }

    pub(crate) fn get(&self) -> &'static T {
        unsafe {
            core::mem::transmute(BUFFER[self.buffer_index * BLOCK_SIZE..(self.buffer_index + 1) * BLOCK_SIZE].as_ptr())
        }
    }
}

impl<T> Drop for Block<T> {
    fn drop(&mut self) {
        unsafe {
            BUFFER_MODE[self.buffer_index] = self.old_buffer_mode;
        }
    }
}

#[inline(always)]
fn is_locked(buffer_index: usize) -> bool {
    unsafe { BUFFER_MODE[buffer_index] == DataMode::Locked }
}

#[inline(always)]
fn read_required(buffer_index: usize, sector: u32) -> bool {
    unsafe { BUFFER_MODE[buffer_index] != DataMode::Read || SECTOR_IN_BUFFER[buffer_index] != sector }
}

impl<CSPIN: PinOps> SdCard<CSPIN> {
    pub(crate) fn read_sector_as<T>(&mut self, buffer_index: usize, sector: u32) -> Result<Block<T>, SdCardError> {
        if buffer_index > BUFFER_COUNT {
            panic!();
        }
        if is_locked(buffer_index) {
            return Err(SdCardError::DataBufferLocked);
        } else if read_required(buffer_index, sector) {
            self.select();
            self.send_card_command(SdCommand::ReadBlock, sector)?;
            unsafe {
                BUFFER_MODE[buffer_index] = DataMode::Read;
                SECTOR_IN_BUFFER[buffer_index] = sector;
                self.read_data(&mut BUFFER[buffer_index * BLOCK_SIZE..(buffer_index + 1) * BLOCK_SIZE])?;
            }
            self.unselect();
        }

        Ok(Block::new(buffer_index))
    }
}
