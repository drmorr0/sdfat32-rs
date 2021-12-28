use super::Volume;
use crate::{
    fat32::{
        constants::*,
        FatError,
        File,
    },
    sdcard::SdCardRef,
};
use avr_hal_generic::port::PinOps;
use core::mem;

const FNAME_FLAG_TRUNCATED: u8 = 0x01;
const FNAME_FLAG_MIXED_CASE: u8 = 0x02;
const FNAME_FLAG_LC_BASE: u8 = 0x08;
const FNAME_FLAG_LC_EXT: u8 = 0x10;

struct DirEntryLFN {
    order: u8,
    unicode1: [u8; 10],
    attributes: u8,
    _always_zero_1: u8,
    checksum: u8,
    unicode2: [u8; 12],
    _always_zero_2: [u8; 2],
    unicode3: [u8; 4],
}

impl DirEntryLFN {
    #[inline(always)]
    fn sequence_num(&self) -> usize {
        (self.order & 0x1f) as usize
    }

    #[inline(always)]
    fn is_last_in_sequence(&self) -> bool {
        self.order & 0x40 > 0
    }
}

pub(crate) struct LFN<'a> {
    path: &'a [u8],
    path_end: usize,
    flags: u8,
    trunc_pos: usize,
    sfn: [u8; 11],
}

impl<'a> LFN<'a> {
    fn new(path: &'a [u8], path_end: usize) -> LFN<'a> {
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
        LFN { path, path_end, flags, trunc_pos, sfn }
    }

    #[inline(always)]
    fn lfn_entry_count(&self) -> usize {
        (self.path_end + 12) / 13
    }
}

pub(crate) fn parse_path_name<'a>(path: &'a [u8]) -> Result<(LFN<'a>, usize), FatError> {
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

    Ok((LFN::new(path, end), pos))
}

impl Volume {
    pub(crate) fn open_file_from_lfn<CSPIN: PinOps>(
        &self,
        sdcard: SdCardRef<CSPIN>,
        dir: &mut File,
        fname: &LFN,
        flags: u8,
    ) -> Result<File, FatError> {
        self.check_dir(dir)?;
        let mut in_lfn_sequence: bool = false;
        let mut expected_checksum: u8 = 0;
        let mut expected_next_sequence_num: usize = 0;
        self.seek(sdcard, dir, 0)?;

        // Try to determine whether the current DirEntry matches up with the provided filename
        for maybe_entry in self.dir_next(sdcard, dir) {
            let entry = maybe_entry?;
            if entry.is_deleted() || entry.is_self_or_parent() {
                in_lfn_sequence = false;
            } else if entry.is_long_name_component() {
                let lfn_entry: DirEntryLFN = unsafe { mem::transmute(entry) };
                let lfn_sequence_num = lfn_entry.sequence_num();

                if !in_lfn_sequence {
                    // We aren't currently in the middle of an LFN sequence

                    // If the sequence number is not what we expect, or this isn't the last entry
                    // in the sequence, this dir-entry can't match fname, so continue
                    if lfn_sequence_num != fname.lfn_entry_count() || !lfn_entry.is_last_in_sequence() {
                        continue;
                    }
                    expected_next_sequence_num = fname.lfn_entry_count();
                    expected_checksum = lfn_entry.checksum;
                    in_lfn_sequence = true;
                } else if lfn_sequence_num != expected_next_sequence_num || lfn_entry.checksum != expected_checksum {
                    // We are in the middle of an LFN sequence, but the order doesn't match or the
                    // checksum doesn't match (I guess this could happen if it was written by
                    // something that supports the spec incorrectly?)
                    in_lfn_sequence = false;
                    continue;
                }
                expected_next_sequence_num -= 1;

                if !compare_lfn_name_segment(&lfn_entry, fname) {
                    in_lfn_sequence = false;
                    continue;
                }
            } else if entry.is_file_or_subdir() {
                // Case 1: This is the "real" entry at the end of a LFN sequence; confirm that
                //         it's what we expected, and then open ze file!
                // Case 2: This is just a regular "short" filename; check if the names match,
                //         and that we're not a long filename in disguise, then open ze file!
                if (in_lfn_sequence && (expected_next_sequence_num == 0) && entry.checksum() == expected_checksum)
                    || (entry.name() == fname.sfn && fname.flags & FNAME_FLAG_TRUNCATED == 0)
                {
                    return Ok(self.open(&entry, flags));
                }
            } else {
                in_lfn_sequence = false;
            }
        }
        Err(FatError::FileNotFound)
    }
}

fn compare_lfn_name_segment(lfn_entry: &DirEntryLFN, fname: &LFN) -> bool {
    for i in 0..13 {
        let c = (get_lfn_char(lfn_entry, i) as char).to_ascii_uppercase();
        let fname_pos = (lfn_entry.sequence_num() - 1) * 13 + i; // LFN entries are 1-indexed
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

fn get_lfn_char(lfn_entry: &DirEntryLFN, i: usize) -> u8 {
    if i < 5 {
        lfn_entry.unicode1[2 * i]
    } else if i < 11 {
        lfn_entry.unicode2[2 * i - 10]
    } else if i < 13 {
        lfn_entry.unicode3[2 * i - 22]
    } else {
        0
    }
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
