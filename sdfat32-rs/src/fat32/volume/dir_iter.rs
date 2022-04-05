use super::{
    DirEntry,
    FatError,
    File,
    SdCardRef,
    Volume,
    LFN,
    SFN,
};
use avr_hal_generic::port::PinOps;
use core::mem;


pub(crate) struct DirectoryIterator<'d, 'v: 'd, 's: 'v, CSPIN: PinOps> {
    dir: &'d mut File,
    lfn_checksum: u8,
    lfn_next: usize,
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
        loop {
            match self.vol.load_sector_for_file::<_, [SFN; 16]>(self.sdcard, self.dir) {
                Ok((entries_raw, sector_pos)) => {
                    let entry_index = sector_pos >> 5; // Divide by 32 to get the index into the array
                    let entry = entries_raw.get()[entry_index];

                    if entry.name()[0] == 0x0 {
                        return None;
                    } else if entry.is_long_name_component() {
                        let lfn_entry: LFN = unsafe { mem::transmute(entry) };

                        if self.lfn_checksum != 0
                            && (self.lfn_checksum != lfn_entry.checksum() || self.lfn_next != lfn_entry.sequence_num())
                        {
                            return Some(Err(FatError::LfnParseError));
                        }

                        // This is a bit confusing.  LFN components are laid out (logically) in
                        // reverse order, so if we want to do something like "read the long name
                        // without having to store a bunch of data or re-read a bunch of data" we
                        // need to process them in reverse.
                        //
                        // When we hit the last logical (first physical) entry in a long filename,
                        // we use the sequence number to skip over the remaining entries.  If we
                        // _haven't_ already processed the LFN entries, then we start walking them
                        // physically backwards (i.e., decreasing dir.pos, but increasing LFN
                        // sequence number).  The second time we hit the last logical entry, then
                        // we know that we can skip ahead to the SFN entry that follows.
                        //
                        // We know whether we've hit the last logical entry before based on whether
                        // we're currently storing a checksum to compare against or not.
                        if lfn_entry.is_last_in_sequence() {
                            self.dir.pos += (lfn_entry.sequence_num() as u32) * 32;
                            if self.lfn_checksum == 0 {
                                self.lfn_checksum = lfn_entry.checksum();
                                self.lfn_next = 1; // LFN sequence numbers are 1-indexed
                                self.dir.pos -= 32;
                                continue;
                            }
                        } else {
                            self.dir.pos -= 32;
                            self.lfn_next += 1;
                        }
                        return Some(Ok(DirEntry::Long(lfn_entry)));
                    } else {
                        if self.lfn_checksum != 0 {
                            // we're done processing an LFN now so clear the relevant data
                            if self.lfn_checksum != entry.checksum() {
                                return Some(Err(FatError::LfnParseError));
                            }
                            self.lfn_checksum = 0;
                            self.lfn_next = 0;
                        }
                        self.dir.pos += 32;
                        return Some(Ok(DirEntry::Short(entry)));
                    }
                },
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

impl Volume {
    pub(crate) fn dir_next<'d, 'v: 'd, 's: 'v, CSPIN: PinOps>(
        &'v self,
        sdcard: SdCardRef<'s, CSPIN>,
        dir: &'d mut File,
    ) -> DirectoryIterator<'s, 'd, 'v, CSPIN> {
        // Callers should ensure that `dir` is a directory
        DirectoryIterator {
            dir,
            lfn_checksum: 0,
            lfn_next: 0,
            sdcard,
            vol: self,
        }
    }
}
