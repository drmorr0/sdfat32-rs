use super::FatError;
use crate::{
    fat32::{
        file::File,
        mbr,
        partition::Partition,
    },
    sdcard::SdCard,
};
use core::cell::RefCell;

pub struct Volume {
    cwd: File,
    pub partition: Partition,
}

impl Volume {
    pub fn read<CSPIN: avr_hal_generic::port::PinOps>(
        sdcard: &RefCell<SdCard<CSPIN>>,
        part_info: &mbr::PartitionInfo,
    ) -> Result<Volume, FatError> {
        unsafe {
            llvm_asm!("nop");
        }
        let partition = Partition::read(sdcard, part_info)?;
        Ok(Volume {
            cwd: File::root_directory(),
            partition,
        })
    }

    pub fn ls() {}

    pub fn seek_file(&self, file: &mut File, pos: u32) -> Result<(), FatError> {
        let current_pos_old = file.current_pos;
        match (|| {
            if !file.is_open() {
                return Err(FatError::FileClosed);
            } else if pos == file.current_pos {
                return Ok(());
            } else if file.is_file() && pos > file.size {
                return Err(FatError::SeekError);
            }

            let mut cluster_idx_new = (pos - 1) >> self.partition.log2_bytes_per_cluster();
            if file.is_contiguous() {
                file.current_cluster = file.first_cluster + cluster_idx_new;
                return Ok(());
            }

            let cluster_idx_cur = (file.current_pos - 1) >> self.partition.log2_bytes_per_cluster();
            if cluster_idx_new < cluster_idx_cur || file.current_pos == 0 {
                file.current_cluster = if file.is_root() {
                    self.partition.data_start_sector
                } else {
                    file.first_cluster
                };
            } else {
                cluster_idx_new -= cluster_idx_cur;
            }

            while cluster_idx_new > 0 {
                //                self.partition.fat_get(file.current_cluster)?;
            }
            Ok(())
        })() {
            Ok(()) => {
                file.current_pos = pos;
                Ok(())
            },
            Err(e) => {
                file.current_pos = current_pos_old;
                Err(e)
            },
        }
    }
}
