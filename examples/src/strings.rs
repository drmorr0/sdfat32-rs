use atmega_hal::{
    clock::MHz16,
    usart::Usart0,
};
use avr_hal_generic::prelude::*;
use avr_progmem_str::{
    pm_write,
    progmem_str,
};

pub fn mid_write(serial: &mut Usart0<MHz16>, mid: u8) {
    match mid {
        0x01 => {
            pm_write!(serial, "Panasonic (0x01)");
        },
        0x02 => {
            pm_write!(serial, "Toshiba (0x02)");
        },
        0x03 => {
            pm_write!(serial, "SanDisk (0x03)");
        },
        0x1b => {
            pm_write!(serial, "Samsung (0x1b)");
        },
        0x1d => {
            pm_write!(serial, "AData (0x1d)");
        },
        0x27 => {
            pm_write!(serial, "Phison (0x27)");
        },
        0x28 => {
            pm_write!(serial, "Lexar (0x28)");
        },
        0x31 => {
            pm_write!(serial, "Silicon Power (0x31)");
        },
        0x41 => {
            pm_write!(serial, "Kingston (0x41)");
        },
        0x74 => {
            pm_write!(serial, "Transcend (0x74)");
        },
        0x76 => {
            pm_write!(serial, "Patriot (0x76)");
        },
        0x82 => {
            pm_write!(serial, "Sony (0x82)");
        },
        0x9c => {
            pm_write!(serial, "Angelbird (0x9c)");
        },
        _ => {
            pm_write!(serial, "Unknown ({})", mid);
        },
    }
}

pub fn ptype_write(serial: &mut Usart0<MHz16>, ptype: u8) {
    match ptype {
        0x00 => {
            pm_write!(serial, "Empty");
        },
        0x07 => {
            pm_write!(serial, "NTFS");
        },
        0x0b => {
            pm_write!(serial, "FAT32 + CHS");
        },
        0x0c => {
            pm_write!(serial, "FAT32 + LBA");
        },
        0x0f => {
            pm_write!(serial, "Extended Partition + LBA");
        },
        0x82 => {
            pm_write!(serial, "Linux Swap Space");
        },
        0x83 => {
            pm_write!(serial, "Linux File System");
        },
        _ => {
            pm_write!(serial, "Unknown ({})", ptype);
        },
    }
}
