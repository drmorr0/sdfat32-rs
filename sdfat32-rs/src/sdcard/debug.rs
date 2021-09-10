use crate::sdcard::cardinfo::{
    CardId,
    CardSpecificData,
};
use avr_progmem_str::{
    pm_write,
    progmem_str,
};
use ufmt::{
    uDebug,
    uWrite,
    Formatter,
};

pub fn mid_write<W>(out: &mut Formatter<W>, mid: u8) -> Result<(), W::Error>
where
    W: uWrite + ?Sized,
{
    match mid {
        0x01 => {
            pm_write!(out, "Panasonic (0x01)")?;
        },
        0x02 => {
            pm_write!(out, "Toshiba (0x02)")?;
        },
        0x03 => {
            pm_write!(out, "SanDisk (0x03)")?;
        },
        0x1b => {
            pm_write!(out, "Samsung (0x1b)")?;
        },
        0x1d => {
            pm_write!(out, "AData (0x1d)")?;
        },
        0x27 => {
            pm_write!(out, "Phison (0x27)")?;
        },
        0x28 => {
            pm_write!(out, "Lexar (0x28)")?;
        },
        0x31 => {
            pm_write!(out, "Silicon Power (0x31)")?;
        },
        0x41 => {
            pm_write!(out, "Kingston (0x41)")?;
        },
        0x74 => {
            pm_write!(out, "Transcend (0x74)")?;
        },
        0x76 => {
            pm_write!(out, "Patriot (0x76)")?;
        },
        0x82 => {
            pm_write!(out, "Sony (0x82)")?;
        },
        0x9c => {
            pm_write!(out, "Angelbird (0x9c)")?;
        },
        _ => {
            pm_write!(out, "Unknown ({})", mid)?;
        },
    };
    Ok(())
}

impl uDebug for CardId {
    fn fmt<W>(&self, out: &mut Formatter<W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        pm_write!(out, "  Manufacturer ID:  ")?;
        mid_write(out, self.manufacturer_id())?;
        out.write_char('\n')?;
        pm_write!(
            out,
            "  OEM ID:           {}{}\n",
            self.oem_id().0 as char,
            self.oem_id().1 as char
        )?;
        pm_write!(out, "  Product name:     ")?;
        for i in 0..5 {
            out.write_char(self.product_name()[i] as char)?;
        }
        out.write_char('\n')?;
        pm_write!(
            out,
            "  Product revision: {}.{}\n",
            self.product_revision().0,
            self.product_revision().1,
        )?;
        pm_write!(
            out,
            "  Serial number:    {}{}\n",
            (self.product_serial_num() >> 16),
            self.product_serial_num(),
        )?;
        pm_write!(
            out,
            "  Manufacture date: {}-{}\n",
            self.manufacturing_date().0,
            self.manufacturing_date().1,
        )?;
        Ok(())
    }
}

impl uDebug for CardSpecificData {
    fn fmt<W>(&self, out: &mut Formatter<W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        pm_write!(out, "  CSD version:               {}\n", self.version())?;
        pm_write!(out, "  Max data transfer rate:    {} MHz\n", self.tran_speed_mhz())?;
        pm_write!(out, "  Supported command classes: ")?;
        for i in 0..12 {
            out.write_char((((self.supported_command_classes() >> (11 - i)) & 0x01) as u8 + b'0') as char)?;
        }
        out.write_char('\n')?;
        pm_write!(
            out,
            "  Max data read block size:  {}\n",
            self.max_read_block_len_bytes(),
        )?;
        pm_write!(out, "  Card capacity:             {} MiB\n", self.capacity_mib())?;
        Ok(())
    }
}
