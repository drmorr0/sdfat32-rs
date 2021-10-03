mod cardinfo;
mod cmd;
mod constants;
mod crc;
mod debug;
mod init;
mod rwdata;
mod sdcard;

pub use constants::BLOCK_SIZE;
pub(crate) use rwdata::{
    Block,
    DATA_BUFFER,
    FS_BUFFER,
};
pub use sdcard::{
    SdCard,
    SdCardRef,
    SdVersion,
};

pub enum SdCardError {
    NoResponse = 1,
    EraseReset,
    IllegalCommand,
    CRCError,
    EraseSequenceError,
    AddressError,
    ParameterError,
    RegisterError,
    ReadError,
    SDVersionOneUnsupported,
    CardCheckPatternMismatch,
    DataBufferLocked,
    Timeout,
    Unknown,
}
