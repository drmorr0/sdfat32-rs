use super::{
    constants::*,
    FatError,
};
use crate::{
    fat32::{
        file::File,
        mbr,
        partition::Partition,
    },
    sdcard::SdCard,
};
use avr_hal_generic::port::PinOps;
use core::{
    cell::RefCell,
    cmp::min,
};

pub struct Volume {
    id: u8,
    pub partition: Partition,
}

impl Volume {
    pub fn open_volume<CSPIN: PinOps>(
        sdcard: &RefCell<SdCard<CSPIN>>,
        part_id: u8,
        part_info: &mbr::PartitionInfo,
    ) -> Result<Volume, FatError> {
        unsafe {
            llvm_asm!("nop");
        }
        let partition = Partition::read(sdcard, part_info)?;
        Ok(Volume { id: part_id, partition })
    }

    pub fn ls<CSPIN: PinOps>(&self, sdcard: &RefCell<SdCard<CSPIN>>, dir: &mut File) -> Result<(), FatError> {
        if !dir.is_directory() {
            return Err(FatError::NotADirectory);
        }
        self.seek_file(sdcard, dir, 0)?;

        for maybe_file in dir.dir_next()? {
            let file = maybe_file?;
            if file.is_contiguous() {
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn open_root(&self) -> File {
        File::open_root(self.id)
    }

    pub fn read_file<CSPIN: PinOps>(
        &self,
        sdcard: &RefCell<SdCard<CSPIN>>,
        file: &mut File,
        buffer: &mut [u8],
    ) -> Result<u32, FatError> {
        if !file.is_readable() {
            return Err(FatError::ReadError);
        }

        let mut buf_pos: usize = 0;

        let bytes_to_eof = file.size() - file.pos;
        let num_bytes = if buffer.len() as u32 > bytes_to_eof {
            bytes_to_eof
        } else {
            buffer.len() as u32
        };

        let mut remainder = num_bytes;
        while remainder > 0 {
            let sector_pos = file.pos & (SECTOR_MASK as u32);
            let sector_of_cluster = self.partition.sector_of_cluster(file.pos);

            // This is the start of a new cluster, but we don't know which one yet
            if sector_pos == 0 && sector_of_cluster == 0 {
                // SdFat has a check for file.pos == 0 here, and sets the cluster accordingly,
                // but I don't think that should be possible in this loop?  If file.pos == 0,
                // it's because a) the file was just opened or b) we called seek, and in either
                // case file.cluster should be set correctly

                if file.is_file() && file.is_contiguous() {
                    file.cluster += 1;
                } else {
                    file.cluster = self.partition.fat_get_next_cluster(sdcard, file.cluster)?;
                }
            }
            let sector = self.partition.cluster_start_sector(file.cluster) + sector_of_cluster;

            let n = if sector_pos != 0 || remainder < BYTES_PER_SECTOR {
                min(BYTES_PER_SECTOR - sector_pos, remainder)
            } else {
                BYTES_PER_SECTOR
            };

            sdcard
                .borrow_mut()
                .read_sectors(sector, &mut buffer[buf_pos..buf_pos + (n as usize)])?;
            buf_pos += n as usize;
            file.pos += n;
            remainder -= n;
        }

        Ok(num_bytes - remainder)
    }

    pub fn seek_file<CSPIN: PinOps>(
        &self,
        sdcard: &RefCell<SdCard<CSPIN>>,
        file: &mut File,
        pos: u32,
    ) -> Result<(), FatError> {
        if file.vol_id != self.id {
            return Err(FatError::VolumeIdMismatch);
        }

        let old_cluster = file.cluster;
        match (|| {
            if !file.is_open() {
                return Err(FatError::FileClosed);
            } else if pos == file.pos {
                return Ok(());
            } else if file.is_file() && pos > file.size() {
                return Err(FatError::SeekError);
            }

            let mut cluster_idx_new = (pos - 1) >> self.partition.log2_bytes_per_cluster();
            if file.is_contiguous() {
                file.cluster = file.start_cluster + cluster_idx_new;
                return Ok(());
            }

            let cluster_idx_cur = (file.pos - 1) >> self.partition.log2_bytes_per_cluster();
            if cluster_idx_new < cluster_idx_cur || file.pos == 0 {
                file.cluster = if file.is_root() {
                    self.partition.data_start_sector
                } else {
                    file.start_cluster
                };
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
}
