use crate::{
    fat32::{
        FatError,
        File,
        Volume,
    },
    sdcard::SdCardRef,
};
use avr_hal_generic::port::PinOps;

pub struct DirEntry {
    pub name: [u8; 11],
    attributes: u8,
    case_flags: u8,
    creation_time_ms: u8,
    creation_time: u16,
    creation_date: u16,
    access_date: u16,
    first_cluster_high: u16,
    modify_time: u16,
    modify_date: u16,
    first_cluster_low: u16,
    file_size: u32,
}

impl DirEntry {
    fn new() -> DirEntry {
        DirEntry {
            name: [0; 11],
            attributes: 0,
            case_flags: 0,
            creation_time_ms: 0,
            creation_time: 0,
            creation_date: 0,
            access_date: 0,
            first_cluster_high: 0,
            modify_time: 0,
            modify_date: 0,
            first_cluster_low: 0,
            file_size: 0,
        }
    }
}

pub struct DirectoryIterator<'d, 'v: 'd, 's: 'v, CSPIN: PinOps> {
    flags: u8,
    dir: &'d mut File,
    sdcard: SdCardRef<'s, CSPIN>,
    vol: &'v mut Volume,
}

impl<CSPIN: PinOps> Iterator for DirectoryIterator<'_, '_, '_, CSPIN> {
    type Item = Result<DirEntry, FatError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Fat directory entries are on 32-byte boundaries
        if self.dir.pos & 0x1f > 0 {
            return Some(Err(FatError::InvalidPosition));
        }
        match self.vol.load_sector_for_file(self.sdcard, self.dir) {
            Ok((sector, sector_pos)) => {
                let mut entry = DirEntry {
                    name: [
                        sector[sector_pos + 0],
                        sector[sector_pos + 1],
                        sector[sector_pos + 2],
                        sector[sector_pos + 3],
                        sector[sector_pos + 4],
                        sector[sector_pos + 5],
                        sector[sector_pos + 6],
                        sector[sector_pos + 7],
                        sector[sector_pos + 7],
                        sector[sector_pos + 9],
                        sector[sector_pos + 10],
                    ],
                    attributes: sector[sector_pos + 11],
                    case_flags: sector[sector_pos + 12],
                    creation_time_ms: sector[sector_pos + 13],
                    creation_time: u16::from_le_bytes([sector[sector_pos + 14], sector[sector_pos + 15]]),
                    creation_date: u16::from_le_bytes([sector[sector_pos + 16], sector[sector_pos + 17]]),
                    access_date: u16::from_le_bytes([sector[sector_pos + 18], sector[sector_pos + 19]]),
                    first_cluster_high: u16::from_le_bytes([sector[sector_pos + 20], sector[sector_pos + 22]]),
                    modify_time: u16::from_le_bytes([sector[sector_pos + 22], sector[sector_pos + 23]]),
                    modify_date: u16::from_le_bytes([sector[sector_pos + 24], sector[sector_pos + 25]]),
                    first_cluster_low: u16::from_le_bytes([sector[sector_pos + 26], sector[sector_pos + 27]]),
                    file_size: u32::from_le_bytes([
                        sector[sector_pos + 28],
                        sector[sector_pos + 29],
                        sector[sector_pos + 30],
                        sector[sector_pos + 31],
                    ]),
                };
                self.dir.pos += 32;

                if entry.name[0] == 0x0 {
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
        &'v mut self,
        sdcard: SdCardRef<'s, CSPIN>,
        dir: &'d mut File,
    ) -> Result<DirectoryIterator<'s, 'd, 'v, CSPIN>, FatError> {
        if !dir.is_directory() {
            return Err(FatError::NotADirectory);
        }

        Ok(DirectoryIterator {
            flags: 0,
            dir,
            sdcard,
            vol: self,
        })
    }
}
