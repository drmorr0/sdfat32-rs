use super::{
    mbr,
    FatError,
};
use crate::sdcard::SdCard;
use core::cell::RefCell;

const LOG2_BYTES_PER_SECTOR: u8 = 9;
const BYTES_PER_SECTOR: u16 = 512;
const SECTOR_MASK: u16 = 0x1FF;

#[repr(packed)]
struct BiosParameterBlock {
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sector_count: u16,
    fat_count: u8,
    _root_dir_entry_count: u16, // Unused in FAT32
    _total_sectors_16: u16,     // Unused in FAT32
    _media_type: u8,
    _sectors_per_fat_16: u16, // Unused in FAT32
    _sectors_per_track: u16,
    _head_count: u16,
    _hidden_sectors: u32,
    total_sectors_32: u32,
    sectors_per_fat_32: u32,
    _fat_32_flags: u16,
    _fat_32_version: u16,
    _fat_32_root_cluster: u32,
    _fat_32_fs_info_sector: u16,
    _fat_32_back_boot_sector: u16,
    _fat_32_reserved: [u8; 12],
    _physical_drive_number: u8,
    _ext_reserved: u8,
    _ext_signature: u8,
    _volume_serial_number: u32,
    volume_label: [u8; 11],
    _volume_type: [u8; 8],
}

impl BiosParameterBlock {
    fn new() -> BiosParameterBlock {
        BiosParameterBlock {
            bytes_per_sector: 0,
            sectors_per_cluster: 0,
            reserved_sector_count: 0,
            fat_count: 0,
            _root_dir_entry_count: 0, // Unused in FAT32
            _total_sectors_16: 0,     // Unused in FAT32
            _media_type: 0,
            _sectors_per_fat_16: 0, // Unused in FAT32
            _sectors_per_track: 0,
            _head_count: 0,
            _hidden_sectors: 0,
            total_sectors_32: 0,
            sectors_per_fat_32: 0,
            _fat_32_flags: 0,
            _fat_32_version: 0,
            _fat_32_root_cluster: 0,
            _fat_32_fs_info_sector: 0,
            _fat_32_back_boot_sector: 0,
            _fat_32_reserved: [0; 12],
            _physical_drive_number: 0,
            _ext_reserved: 0,
            _ext_signature: 0,
            _volume_serial_number: 0,
            volume_label: [0; 11],
            _volume_type: [0; 8],
        }
    }
}

#[repr(packed)]
struct PartitionBootSector {
    _jump_instr: [u8; 3],
    _oem_name: [u8; 8],
    bios_params: BiosParameterBlock,
    _boot_code: [u8; 390],
    _signature: [u8; 2],
}

pub struct Partition {
    pub(crate) alloc_search_start: u32,
    pub(crate) cluster_sector_mask: u8,
    pub(crate) data_cluster_count: u32,
    pub(crate) data_start_sector: u32,
    pub(crate) fat_start_sector: u32,
    pub(crate) free_cluster_count: Option<u32>,
    pub(crate) log2_sectors_per_cluster: u8,
    pub(crate) sectors_per_cluster: u8,
    pub(crate) sectors_per_fat: u32,
    pub(crate) volume_label: [u8; 11],
}

impl Partition {
    pub(crate) fn read<CSPIN: avr_hal_generic::port::PinOps>(
        sdcard: &RefCell<SdCard<CSPIN>>,
        partition_info: &mbr::PartitionInfo,
    ) -> Result<Partition, FatError> {
        let mut pbs: PartitionBootSector = PartitionBootSector {
            _jump_instr: [0; 3],
            _oem_name: [0; 8],
            bios_params: BiosParameterBlock::new(),
            _boot_code: [0; 390],
            _signature: [0; 2],
        };
        let raw_pbs =
            unsafe { core::slice::from_raw_parts_mut((&mut pbs as *mut PartitionBootSector) as *mut u8, 512) };
        if let Err(_) = sdcard.borrow_mut().read_sectors(partition_info.start_sector, raw_pbs) {
            return Err(FatError::CorruptPartition);
        }
        let bp = &pbs.bios_params;

        if bp.fat_count != 2 || bp.bytes_per_sector != BYTES_PER_SECTOR {
            return Err(FatError::CorruptPartition);
        }

        let mut log2_sectors_per_cluster: u8 = 0;
        let mut i = 1;
        while i != bp.sectors_per_cluster {
            if i == 0 {
                return Err(FatError::CorruptPartition);
            }
            log2_sectors_per_cluster += 1;
            i <<= 1;
        }
        let sectors_per_fat = bp.sectors_per_fat_32;
        let fat_start_sector = partition_info.start_sector + bp.reserved_sector_count as u32;
        let data_start_sector = fat_start_sector + (bp.fat_count as u32) * sectors_per_fat;
        let mut data_cluster_count = bp.total_sectors_32 - (data_start_sector - partition_info.start_sector);
        data_cluster_count >>= log2_sectors_per_cluster;

        if data_cluster_count < 65525 {
            return Err(FatError::Unsupported);
        }

        Ok(Partition {
            alloc_search_start: 1,
            cluster_sector_mask: bp.sectors_per_cluster - 1,
            data_cluster_count,
            data_start_sector,
            fat_start_sector,
            free_cluster_count: None, // Unknown number of free clusters
            log2_sectors_per_cluster,
            sectors_per_cluster: bp.sectors_per_cluster,
            sectors_per_fat,
            volume_label: bp.volume_label,
        })
    }

    pub(crate) fn log2_bytes_per_cluster(&self) -> u8 {
        // Operating in log space so multiplication becomes addition
        self.log2_sectors_per_cluster + LOG2_BYTES_PER_SECTOR
    }
}
