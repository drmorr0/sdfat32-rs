use super::{
    DirEntry,
    FatError,
    File,
    SdCardRef,
    Volume,
    LFN,
};
use crate::fat32::constants::*;
use avr_hal_generic::port::PinOps;

const FNAME_FLAG_TRUNCATED: u8 = 0x01;
const FNAME_FLAG_MIXED_CASE: u8 = 0x02;
const FNAME_FLAG_LC_BASE: u8 = 0x08;
const FNAME_FLAG_LC_EXT: u8 = 0x10;


pub(crate) struct Fname<'a> {
    path: &'a [u8],
    path_end: usize,
    flags: u8,
    trunc_pos: usize,
    sfn: [u8; 11],
}

impl<'a> Fname<'a> {
    fn new(path: &'a [u8], path_end: usize) -> Fname<'a> {
        let mut flags: u8 = 0;
        let mut trunc_pos: usize = 0;
        let mut sfn: [u8; 11] = [SPACE; 11];
        let (mut sfn_pos, mut sfn_end) = (0usize, 7usize);
        let (mut path_pos, mut last_dot_pos) = (0usize, path_end - 1);

        let (mut lc_parts, mut uc_parts) = (0u8, 0u8);
        let mut is83 = true; // Is the original filename a valid 8.3 filename?
        let mut in_ext = false;

        // Can't start with a dot...
        while path[path_pos] == DOT {
            path_pos += 1;
            is83 = false;
        }

        // Find the last dot...
        while path[last_dot_pos] != DOT && last_dot_pos > path_pos {
            last_dot_pos -= 1;
        }

        'outer: while path_pos < path_end {
            loop {
                let mut c = path[path_pos];
                if c == DOT && path_pos == last_dot_pos {
                    // We're in the extension now
                    sfn_pos = 8;
                    sfn_end = 10;
                    in_ext = true;
                } else {
                    if c == SPACE || c == DOT {
                        // Skip spaces and periods (which the original SdFat lib doesn't seem to do...?)
                        is83 = false;
                        break;
                    } else if sfn_reserved_char(c) {
                        // Replace reserved characters with underscores in the SFN
                        is83 = false;
                        // Skip UTF-8 trailing characters (I don't know what this means)
                        if (c & 0xc0) == 0x80 {
                            continue;
                        }
                        c = '_' as u8;
                    }

                    if sfn_pos > sfn_end {
                        is83 = false;
                        if in_ext || path_pos > last_dot_pos {
                            // all done; either the extension's longer than three characters
                            // or there is no extension
                            break 'outer;
                        }
                        // skip to the last dot (i.e., the start of the extension)
                        path_pos = last_dot_pos - 1;
                        break;
                    }

                    if (c as char).is_lowercase() {
                        c -= 32; // Offset between 'A' and 'a' in the ASCII table
                        lc_parts |= if in_ext { FNAME_FLAG_LC_EXT } else { FNAME_FLAG_LC_BASE }
                    } else {
                        uc_parts |= if in_ext { FNAME_FLAG_LC_EXT } else { FNAME_FLAG_LC_BASE }
                    }

                    sfn[sfn_pos] = c;
                    sfn_pos += 1;
                    if sfn_pos < 7 {
                        trunc_pos = sfn_pos;
                    }
                }
                break;
            }
            path_pos += 1;
        }

        if is83 {
            flags = if lc_parts != uc_parts { FNAME_FLAG_MIXED_CASE } else { lc_parts };
        } else {
            flags = FNAME_FLAG_TRUNCATED;
            sfn[trunc_pos] = '~' as u8;
            sfn[trunc_pos + 1] = '1' as u8;
        }
        Fname { path, path_end, flags, trunc_pos, sfn }
    }

    pub(crate) fn checksum(&self) -> u8 {
        let mut sum: u8 = 0;
        for i in 0..self.path_end {
            let c = self.path[i];
            sum = (((sum & 1) << 7) | (sum >> 1)) + c;
        }
        sum
    }

    #[inline(always)]
    fn lfn_entry_count(&self) -> usize {
        (self.path_end + 12) / 13
    }
}

pub(crate) fn parse_path_name<'a>(path: &'a [u8]) -> Result<(Fname<'a>, usize), FatError> {
    let mut pos = 0;
    let mut end = 0;
    while pos < path.len() && path[pos] > 0 && path[pos] != DIR_SEPARATOR {
        if path[pos] >= 0x80 || lfn_reserved_char(path[pos]) {
            return Err(FatError::ParsePathError);
        }
        if path[pos] != DOT && path[pos] != SPACE {
            end = pos + 1;
        }
        pos += 1;
    }

    if end == 0 || end > MAX_LFN_LEN {
        return Err(FatError::ParsePathError);
    }

    // Skip to the start of the next component of the path; we broke the last loop when we
    // a) hit the end, or b) saw a separator, so we're still at a separator here
    while pos < path.len() && path[pos] > 0 && (path[pos] == SPACE || path[pos] == DIR_SEPARATOR) {
        pos += 1;
    }

    Ok((Fname::new(path, end), pos))
}

impl Volume {
    pub(crate) fn open_file_from_lfn<CSPIN: PinOps>(
        &self,
        sdcard: SdCardRef<CSPIN>,
        dir: &mut File,
        fname: &Fname,
        flags: u8,
    ) -> Result<File, FatError> {
        self.check_dir(dir)?;
        self.seek(sdcard, dir, 0)?;

        // Try to determine whether the current DirEntry matches up with the provided filename
        for maybe_entry in self.dir_next(sdcard, dir) {
            let entry = maybe_entry?;
            match entry {
                DirEntry::Long(lfn) => {
                    // if fname.checksum() != lfn.checksum() || !compare_lfn_name_segment(&lfn,
                    // fname) {     continue;
                    // }
                },
                DirEntry::Short(sfn) => {
                    // Case 1: This is the "real" entry at the end of a LFN sequence; confirm that
                    //         it's what we expected, and then open ze file!
                    // Case 2: This is just a regular "short" filename; check if the names match,
                    //         and that we're not a long filename in disguise, then open ze file!
                    // if sfn.checksum() == fname.checksum()
                    //     && sfn.name() == fname.sfn
                    //     && fname.flags & FNAME_FLAG_TRUNCATED == 0
                    if sfn.name() == fname.sfn && fname.flags & FNAME_FLAG_TRUNCATED == 0 {
                        return Ok(self.open(&sfn, flags));
                    }
                },
            }
        }
        Err(FatError::FileNotFound)
    }
}

fn compare_lfn_name_segment(lfn: &LFN, fname: &Fname) -> bool {
    for i in 0..13 {
        let c = (lfn.get_char(i) as char).to_ascii_uppercase();
        let fname_pos = (lfn.sequence_num() as usize - 1) * 13 + i; // LFN entries are 1-indexed
        let fname_c = (fname.path[fname_pos] as char).to_ascii_uppercase();

        // LFN entries and fname entries should be zero-terminated, so check one past the end
        if fname_pos > fname.path_end {
            break;
        } else if c != fname_c {
            return false;
        }
    }
    return true;
}

#[inline(always)]
fn sfn_reserved_char(c: u8) -> bool {
    // ", [, \, ], |, *+,./, :;<=>?,
    return c < 0x20
        || c > 0x7f
        || c == 0x5b
        || c == 0x5c
        || c == 0x5d
        || c == 0x7c
        || (0x2a <= c && c <= 0x2f && c != 0x2d)
        || (0x3a <= c && c <= 0x3f);
}

#[inline(always)]
fn lfn_reserved_char(c: u8) -> bool {
    // ", *, /, :, <, >, ?, \, |
    return c < 0x20
        || c == 0x22
        || c == 0x2a
        || c == 0x2f
        || c == 0x3a
        || c == 0x3c
        || c == 0x3e
        || c == 0x3f
        || c == 0x5c
        || c == 0x7c;
}
