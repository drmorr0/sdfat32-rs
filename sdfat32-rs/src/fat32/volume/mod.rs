mod dir_iter;

use super::{
    constants::*,
    FatError,
};
use crate::{
    fat32::{
        mbr,
        partition::Partition,
        DirEntry,
        File,
    },
    sdcard::{
        Block,
        SdCardRef,
        DATA_BUFFER,
    },
};
use avr_hal_generic::port::PinOps;
use core::{
    cmp::min,
    convert::TryInto,
};


pub struct Volume {
    pub partition: Partition,
    id: u8,
}

impl Volume {
    pub fn open_volume<CSPIN: PinOps>(
        sdcard: SdCardRef<CSPIN>,
        part_id: u8,
        part_info: &mbr::PartitionInfo,
    ) -> Result<Volume, FatError> {
        Ok(Volume {
            partition: Partition::read(sdcard, part_info)?,
            id: part_id,
        })
    }

    pub fn ls<CSPIN: PinOps, T>(
        &self,
        sdcard: SdCardRef<CSPIN>,
        dir: &mut File,
        depth: u16,
        depth_limit: u16,
        context: &mut T,
        mut func: impl FnMut(&DirEntry, u16, &mut T) -> () + Copy,
    ) -> Result<(), FatError> {
        if dir.vol_id != self.id {
            return Err(FatError::VolumeIdMismatch);
        } else if !dir.is_directory() {
            return Err(FatError::NotADirectory);
        }
        self.seek_file(sdcard, dir, 0)?;

        for maybe_file_entry in self.dir_next(sdcard, dir)? {
            let file_entry = maybe_file_entry?;
            func(&file_entry, depth, context);
            if depth_limit > 0 && file_entry.is_directory() && !file_entry.is_self_or_parent() {
                let mut d = self.open(&file_entry);
                self.ls(sdcard, &mut d, depth + 1, depth_limit - 1, context, func)?
            }
        }
        Ok(())
    }

    pub fn open(&self, entry: &DirEntry) -> File {
        File::open(self.id, entry)
    }

    pub fn open_root(&self) -> File {
        File::open_root(self.id)
    }

    // pub fn read_file<CSPIN: PinOps>(
    //     &mut self,
    //     sdcard: SdCardRef<CSPIN>,
    //     file: &mut File,
    //     buffer: &mut [u8],
    // ) -> Result<usize, FatError> {
    //     if file.vol_id != self.id {
    //         return Err(FatError::VolumeIdMismatch);
    //     } else if !file.is_readable() {
    //         return Err(FatError::ReadError);
    //     }

    //     let mut buf_pos: usize = 0;

    //     // We can't read in more than usize bytes so it's fine if num_bytes is a usize, not a u32
    //     let bytes_to_eof = file.size() - file.pos;
    //     let num_bytes: usize = if buffer.len() as u32 > bytes_to_eof {
    //         bytes_to_eof.try_into().unwrap()
    //     } else {
    //         buffer.len()
    //     };

    //     let mut remainder = num_bytes;
    //     while remainder > 0 {
    //         let (sector, sector_pos) = self.load_sector_for_file(sdcard, file)?;
    //         let n: usize = if sector_pos != 0 || remainder < BYTES_PER_SECTOR {
    //             // Safe to do this cast because the max value is BYTES_PER_SECTOR
    //             min(BYTES_PER_SECTOR - sector_pos, remainder)
    //         } else {
    //             BYTES_PER_SECTOR
    //         };

    //         buffer[buf_pos..buf_pos + n].copy_from_slice(&sector[sector_pos..sector_pos + n]);

    //         buf_pos += n;
    //         file.pos += n as u32;
    //         remainder -= n;
    //     }

    //     Ok(num_bytes - remainder)
    // }

    pub fn seek_file<CSPIN: PinOps>(
        &self,
        sdcard: SdCardRef<CSPIN>,
        file: &mut File,
        pos: u32,
    ) -> Result<(), FatError> {
        if file.vol_id != self.id {
            return Err(FatError::VolumeIdMismatch);
        } else if !file.is_open() {
            return Err(FatError::FileClosed);
        } else if pos == file.pos {
            return Ok(());
        } else if file.is_file() && pos > file.size() {
            return Err(FatError::SeekError);
        }

        let old_cluster = file.cluster;
        match (|| {
            let mut cluster_idx_new = (pos - 1) >> self.partition.log2_bytes_per_cluster();
            if file.is_contiguous() {
                file.cluster = file.start_cluster + cluster_idx_new;
                return Ok(());
            }

            let cluster_idx_cur = (file.pos - 1) >> self.partition.log2_bytes_per_cluster();
            if cluster_idx_new < cluster_idx_cur || file.pos == 0 {
                file.cluster = if file.is_root() { ROOT_CLUSTER } else { file.start_cluster };
            } else {
                cluster_idx_new -= cluster_idx_cur;
            }

            for _ in 0..cluster_idx_new {
                file.cluster = self.partition.fat_get_next_cluster(sdcard, file.cluster)?;
            }
            Ok(())
        })() {
            Ok(()) => {
                file.pos = pos;
                Ok(())
            },
            Err(e) => {
                file.cluster = old_cluster;
                Err(e)
            },
        }
    }

    // returns: the position in the sector corresponding to the file.pos
    // (guaranteed to be at most BYTES_PER_SECTOR, so usize is fine)
    fn load_sector_for_file<CSPIN: PinOps, T>(
        &self,
        sdcard: SdCardRef<CSPIN>,
        file: &mut File,
    ) -> Result<(Block<T>, usize), FatError> {
        // Unchecked; we assume that the file belongs to this volume and is readable
        let sector_pos = (file.pos & (SECTOR_MASK as u32)) as usize;
        let sector_of_cluster = self.partition.sector_of_cluster(file.pos);

        // This is the start of a new cluster, but we don't know which one yet
        if file.pos != 0 && sector_pos == 0 && sector_of_cluster == 0 {
            if file.is_file() && file.is_contiguous() {
                file.cluster += 1;
            } else {
                file.cluster = self.partition.fat_get_next_cluster(sdcard, file.cluster)?;
            }
        }
        let sector_index = self.partition.cluster_start_sector(file.cluster) + sector_of_cluster;
        match sdcard.borrow_mut().read_sector_as::<T>(DATA_BUFFER, sector_index) {
            Ok(sector) => Ok((sector, sector_pos)),
            Err(e) => Err(FatError::from(e)),
        }
    }
}
