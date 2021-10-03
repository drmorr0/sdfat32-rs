use super::{
    constants::*,
    mbr,
    FatError,
};
use crate::sdcard::SdCardRef;
use avr_hal_generic::port::PinOps;
use core::convert::TryInto;


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
    pub(crate) fn read<CSPIN: PinOps>(
        sdcard: SdCardRef<CSPIN>,
        partition_info: &mbr::PartitionInfo,
    ) -> Result<Partition, FatError> {
        let mut sd_borrow_mut = sdcard.borrow_mut();
        let pbs_block = sd_borrow_mut.read_sector_as::<PartitionBootSector>(0, partition_info.start_sector)?;
        let pbs = pbs_block.get();
        let bp = &pbs.bios_params;

        if bp.fat_count != 2 || bp.bytes_per_sector != BYTES_PER_SECTOR as u16 {
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
            return Err(FatError::UnsupportedVersion);
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

    pub(crate) fn fat_get_next_cluster<CSPIN: PinOps>(
        &self,
        sdcard: SdCardRef<CSPIN>,
        cluster: u32,
    ) -> Result<u32, FatError> {
        if cluster < 2 || cluster > self.last_cluster() {
            return Err(FatError::InvalidCluster);
        }

        // Each sector (512 bytes) contains 128 cluster entries since FAT32 entries are 4 bytes
        // long.  So here we divide by 512 / 4 to get the sector index that we're interested in.
        let fat_sector_to_get = self.fat_start_sector + (cluster >> (LOG2_BYTES_PER_SECTOR - 2));

        // TODO implement caching for faster lookups
        let mut sd_borrow_mut = sdcard.borrow_mut();
        let fat_sector_data = sd_borrow_mut.read_sector_as::<SECTOR>(0, fat_sector_to_get)?.get();

        let idx = (cluster & ((self.cluster_sector_mask >> 2) as u32)) as usize;
        let sector_bytes_for_cluster = match fat_sector_data[idx..idx + 4].try_into() {
            Ok(val) => val,
            Err(_) => return Err(FatError::CorruptFat),
        };

        // TODO need EOC check
        Ok(u32::from_le_bytes(sector_bytes_for_cluster))
    }

    #[inline(always)]
    pub(crate) fn cluster_start_sector(&self, cluster: u32) -> u32 {
        // Skip the two reserved clusters at the beginning
        self.data_start_sector + ((cluster - 2) << self.log2_sectors_per_cluster)
    }

    #[inline(always)]
    pub(crate) fn last_cluster(&self) -> u32 {
        self.data_cluster_count + 1
    }

    #[inline(always)]
    pub(crate) fn log2_bytes_per_cluster(&self) -> u8 {
        // Operating in log space so multiplication becomes addition
        self.log2_sectors_per_cluster + LOG2_BYTES_PER_SECTOR
    }

    #[inline(always)]
    pub(crate) fn sector_of_cluster(&self, pos: u32) -> u32 {
        // Divide by the number of sectors per cluster, and mask to restrict to the current cluster
        pos >> self.log2_sectors_per_cluster & (self.cluster_sector_mask as u32)
    }
}
