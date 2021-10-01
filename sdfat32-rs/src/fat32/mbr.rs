use super::FatError;
use crate::sdcard::SdCard;
use core::cell::RefCell;

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct PartitionInfo {
    pub boot: u8,
    pub begin_chs: [u8; 3],
    pub ptype: u8,
    pub end_chs: [u8; 3],

    // AVR is little-endian and so is FAT32 so this is safe
    pub start_sector: u32,
    pub total_sectors: u32,
}

impl PartitionInfo {
    pub(crate) fn new() -> PartitionInfo {
        PartitionInfo {
            boot: 0,
            begin_chs: [0; 3],
            ptype: 0,
            end_chs: [0; 3],
            start_sector: 0,
            total_sectors: 0,
        }
    }
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
            partitions: [
                PartitionInfo::new(),
                PartitionInfo::new(),
                PartitionInfo::new(),
                PartitionInfo::new(),
            ],
            signature: [0; 2],
        }
    }

    pub fn read_part_info<CSPIN: avr_hal_generic::port::PinOps>(
        sdcard: &RefCell<SdCard<CSPIN>>,
    ) -> Result<[PartitionInfo; 4], FatError> {
        let mut sd_borrow_mut = sdcard.borrow_mut();
        let mbr = sd_borrow_mut.read_sector_as::<Mbr>(0)?;
        Ok(mbr.partitions)
    }
}
