use super::{
    mbr::PartitionInfo,
    partition::Partition,
};
use crate::hexfmt::hexfmt32_le;
use avr_progmem_str::{
    pm_write,
    progmem_str,
};
use ufmt::{
    uDebug,
    uWrite,
    Formatter,
};


pub fn ptype_write<W>(serial: &mut Formatter<W>, ptype: u8) -> Result<(), W::Error>
where
    W: uWrite + ?Sized,
{
    match ptype {
        0x00 => pm_write!(serial, "Empty")?,
        0x07 => pm_write!(serial, "NTFS")?,
        0x0b => pm_write!(serial, "FAT32 + CHS")?,
        0x0c => pm_write!(serial, "FAT32 + LBA")?,
        0x0f => pm_write!(serial, "Extended Partition + LBA")?,
        0x82 => pm_write!(serial, "Linux Swap Space")?,
        0x83 => pm_write!(serial, "Linux File System")?,
        _ => pm_write!(serial, "Unknown ({})", ptype)?,
    };
    Ok(())
}

impl uDebug for PartitionInfo {
    fn fmt<W>(&self, out: &mut Formatter<W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        pm_write!(out, "  is_boot = {}; partition_type = ", self.boot)?;
        ptype_write(out, self.ptype)?;
        pm_write!(
            out,
            "; begin_chs = {}/{}/{}; end_chs = {}/{}/{}; start_sector = "
            self.begin_chs[0],
            self.begin_chs[1],
            self.begin_chs[2],
            self.end_chs[0],
            self.end_chs[1],
            self.end_chs[2],
        )?;
        hexfmt32_le(out, self.start_sector)?;
        pm_write!(out, ", length = ")?;
        hexfmt32_le(out, self.total_sectors)?;
        out.write_char('\n')?;
        Ok(())
    }
}

impl uDebug for Partition {
    fn fmt<W>(&self, out: &mut Formatter<W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        pm_write!(out, "  alloc search start:  ")?;
        hexfmt32_le(out, self.alloc_search_start)?;
        out.write_char('\n')?;
        pm_write!(out, "  cluster sector mask: {}\n", self.cluster_sector_mask)?;
        pm_write!(out, "  data cluster count:  ")?;
        hexfmt32_le(out, self.data_cluster_count)?;
        out.write_char('\n')?;
        pm_write!(out, "  data start sector:   ")?;
        hexfmt32_le(out, self.data_start_sector)?;
        out.write_char('\n')?;
        pm_write!(out, "  fat start sector:    ")?;
        hexfmt32_le(out, self.fat_start_sector)?;
        out.write_char('\n')?;
        pm_write!(out, "  sectors per cluster: {}\n", self.sectors_per_cluster)?;
        pm_write!(out, "  sectors per fat:     ")?;
        hexfmt32_le(out, self.sectors_per_fat)?;
        out.write_char('\n')?;
        pm_write!(out, "  volume label:        ")?;
        for c in self.volume_label {
            out.write_char(c as char)?;
        }
        out.write_char('\n')?;
        Ok(())
    }
}
