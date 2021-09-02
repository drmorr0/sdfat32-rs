use super::{
    mbr,
    FatError,
};
use crate::sdcard::SdCard;
use core::default::Default;

const BYTES_PER_SECTOR_SHIFT: u8 = 9;
const BYTES_PER_SECTOR: u16 = 512;
const SECTOR_MASK: u16 = 0x1FF;

#[repr(packed)]
#[derive(Default, Clone, Copy)]
struct BiosParameterBlockFat32 {
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
    _volume_label: [u8; 11],
    _volume_type: [u8; 8],
}

#[repr(packed)]
struct PartitionBootSector {
    _jump_instr: [u8; 3],
    _oem_name: [u8; 8],
    bios_params: BiosParameterBlockFat32,
    _boot_code: [u8; 390],
    _signature: [u8; 2],
}

pub struct Partition {
    pub alloc_search_start: u32,
    pub cluster_count: u32,
    pub cluster_sector_mask: u8,
    pub fat_start_sector: u32,
    pub free_cluster_count: Option<u32>,
    pub root_dir_start: u32,
    pub sectors_per_cluster: u8,
    pub sectors_per_cluster_shift: u8,
    pub sectors_per_fat: u32,
}

impl Partition {
    pub fn read<CSPIN: avr_hal_generic::port::PinOps>(
        sdcard: &mut SdCard<CSPIN>,
        partition_info: &mbr::PartitionInfo,
    ) -> Result<Partition, FatError> {
        let mut pbs: PartitionBootSector = PartitionBootSector {
            _jump_instr: [0; 3],
            _oem_name: [0; 8],
            bios_params: Default::default(),
            _boot_code: [0; 390],
            _signature: [0; 2],
        };
        let raw_pbs =
            unsafe { core::slice::from_raw_parts_mut((&mut pbs as *mut PartitionBootSector) as *mut u8, 512) };
        if let Err(_) = sdcard.read_sectors(partition_info.start_sector, raw_pbs) {
            return Err(FatError::CorruptPartition);
        }
        let bp = &pbs.bios_params;

        if bp.fat_count != 2 || bp.bytes_per_sector != BYTES_PER_SECTOR {
            return Err(FatError::CorruptPartition);
        }

        let mut sectors_per_cluster_shift: u8 = 0;
        let mut i = 1;
        while i != bp.sectors_per_cluster {
            if i == 0 {
                return Err(FatError::CorruptPartition);
            }
            sectors_per_cluster_shift += 1;
            i <<= 1;
        }
        let sectors_per_fat = bp.sectors_per_fat_32;
        let fat_start_sector = partition_info.start_sector + bp.reserved_sector_count as u32;
        let root_dir_start = fat_start_sector + (bp.fat_count as u32) * sectors_per_fat;
        let mut cluster_count = bp.total_sectors_32 - root_dir_start + partition_info.start_sector;
        cluster_count >>= sectors_per_cluster_shift;

        if cluster_count < 65525 {
            return Err(FatError::Unsupported);
        }

        Ok(Partition {
            alloc_search_start: 1,
            cluster_count,
            cluster_sector_mask: bp.sectors_per_cluster - 1,
            fat_start_sector,
            free_cluster_count: None, // Unknown number of free clusters
            root_dir_start,
            sectors_per_cluster: bp.sectors_per_cluster,
            sectors_per_cluster_shift,
            sectors_per_fat,
        })
    }
}
