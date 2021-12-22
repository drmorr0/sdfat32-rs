use crate::{
    fat32::{
        DirEntry,
        FatError,
        File,
        Volume,
    },
    sdcard::SdCardRef,
};
use avr_hal_generic::port::PinOps;


pub struct DirectoryIterator<'d, 'v: 'd, 's: 'v, CSPIN: PinOps> {
    flags: u8,
    dir: &'d mut File,
    sdcard: SdCardRef<'s, CSPIN>,
    vol: &'v Volume,
}

impl<CSPIN: PinOps> Iterator for DirectoryIterator<'_, '_, '_, CSPIN> {
    type Item = Result<DirEntry, FatError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Fat directory entries are on 32-byte boundaries
        if self.dir.pos & 0x1f > 0 {
            return Some(Err(FatError::InvalidPosition));
        }
        match self.vol.load_sector_for_file::<_, [DirEntry; 16]>(self.sdcard, self.dir) {
            Ok((entries_raw, sector_pos)) => {
                let entry_index = sector_pos >> 5;
                let entry = entries_raw.get()[entry_index];
                self.dir.pos += 32;

                if entry.name()[0] == 0x0 {
                    None
                } else {
                    Some(Ok(entry))
                }
            },
            Err(e) => Some(Err(e)),
        }
    }
}

impl Volume {
    pub fn dir_next<'d, 'v: 'd, 's: 'v, CSPIN: PinOps>(
        &'v self,
        sdcard: SdCardRef<'s, CSPIN>,
        dir: &'d mut File,
    ) -> Result<DirectoryIterator<'s, 'd, 'v, CSPIN>, FatError> {
        if !dir.is_directory() {
            return Err(FatError::NotADirectory);
        }

        Ok(DirectoryIterator { flags: 0, dir, sdcard, vol: self })
    }
}
