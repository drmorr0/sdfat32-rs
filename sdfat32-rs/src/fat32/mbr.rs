use super::FatError;
use crate::sdcard::SdCard;
use core::default::Default;

#[repr(packed)]
#[derive(Default, Clone, Copy)]
pub struct PartitionInfo {
    pub boot: u8,
    pub begin_chs: [u8; 3],
    pub ptype: u8,
    pub end_chs: [u8; 3],

    // AVR is little-endian and so is FAT32 so this is safe
    pub start_sector: u32,
    pub total_sectors: u32,
}

#[repr(packed)]
pub struct Mbr {
    pub boot_code: [u8; 446],
    pub partitions: [PartitionInfo; 4],
    pub signature: [u8; 2],
}

impl Mbr {
    pub fn new() -> Mbr {
        Mbr {
            boot_code: [0; 446],
            partitions: [Default::default(); 4],
            signature: [0; 2],
        }
    }
    pub fn read<CSPIN: avr_hal_generic::port::PinOps>(&mut self, sdcard: &mut SdCard<CSPIN>) -> Result<(), FatError> {
        let raw_mbr = unsafe { core::slice::from_raw_parts_mut((self as *mut Mbr) as *mut u8, 512) };
        if let Err(e) = sdcard.read_sectors(0, raw_mbr) {
            return Err(FatError::CorruptMBR);
        }
        Ok(())
    }
}
