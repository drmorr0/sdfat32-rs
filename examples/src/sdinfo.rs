#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(abi_avr_interrupt)]
#![feature(panic_info_message)]
#![allow(unreachable_code)]
#![allow(dead_code)]
#![allow(unused_imports)]

use arduino_hal::{
    prelude::*,
    spi,
};
use atmega_hal::{
    clock::MHz16,
    port::PB5,
    usart::Usart0,
};
use avr_async::{
    init_timers,
    millis,
};
use avr_hal_generic::port::{
    mode::Output,
    Pin,
};
use avr_progmem;
use avr_progmem_str::{
    pm_write,
    progmem_str,
};
use embedded_hal::spi::MODE_0;
use sdfat32_rs::{
    fat32::mbr::MbrSector,
    sdcard::{
        SdCard,
        SdVersion,
    },
};
use ufmt::{
    uwrite,
    uwriteln,
};

fn mid_lookup(mid: u8) -> &'static str {
    match mid {
        0x01 => "Panasonic",
        0x02 => "Toshiba",
        0x03 => "SanDisk",
        0x1b => "Samsung",
        0x1d => "AData",
        0x27 => "Phison",
        0x28 => "Lexar",
        0x31 => "Silicon Power",
        0x41 => "Kingston",
        0x74 => "Transcend",
        0x76 => "Patriot",
        0x82 => "Sony",
        0x9c => "Angelbird",
        _ => "Unknown",
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = match arduino_hal::Peripherals::take() {
        Some(p) => p,
        None => panic!("Aborting"),
    };
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    init_timers();

    let (spi, cs) = arduino_hal::Spi::new(
        dp.SPI,
        pins.d13.into_output(),
        pins.d11.into_output(),
        pins.d12.into_pull_up_input(),
        pins.d10.into_output(),
        spi::Settings {
            data_order: spi::DataOrder::MostSignificantFirst,
            clock: spi::SerialClockRate::OscfOver64,
            mode: MODE_0,
        },
    );

    let mut sdcard = match SdCard::new(spi, cs, millis) {
        Ok(s) => s,
        Err(e) => {
            pm_write!(serial, "SdCard initialization failed with error code {}\n", e as u8);
            panic!("Aborting")
        },
    };

    pm_write!(serial, "\nCardInfo:\n");
    let version_str = match sdcard.version {
        SdVersion::One => "1.0",
        SdVersion::Two { sdhc: true } => "2.0 (SDHC)",
        SdVersion::Two { sdhc: false } => "2.0",
    };
    pm_write!(serial, "  SD Card Version:  {}\n", version_str);

    match sdcard.read_card_id() {
        Ok(cid) => {
            pm_write!(
                serial,
                "  Manufacturer ID:  {} ({})\n",
                mid_lookup(cid.manufacturer_id),
                cid.manufacturer_id,
            );
            pm_write!(serial, "  OEM ID:           {}{}\n", cid.oem_id.0, cid.oem_id.1);
            pm_write!(serial, "  Product name:     ");
            for i in 0..5 {
                uwrite!(serial, "{}", cid.product_name[i]).void_unwrap();
            }
            uwrite!(serial, "\n").void_unwrap();
            pm_write!(
                serial,
                "  Product revision: {}.{}\n",
                cid.product_revision.0,
                cid.product_revision.1,
            );
            pm_write!(
                serial,
                "  Serial number:    {}{}\n",
                (cid.product_serial_num >> 16),
                cid.product_serial_num,
            );
            pm_write!(
                serial,
                "  Manufacture date: {}-{}\n",
                cid.manufacturing_date_year,
                cid.manufacturing_date_month,
            );
        },
        Err(e) => {
            pm_write!(serial, "couldn't read CID register: {}\n", e as u8);
            panic!("Aborting...");
        },
    }
    pm_write!(serial, "\nCard-specific data:\n");
    match sdcard.read_card_specific_data() {
        Ok(csd) => {
            pm_write!(serial, "  CSD version:               {}\n", csd.version);
            pm_write!(serial, "  Max data transfer rate:    {} MHz\n", csd.tran_speed_mhz);
            pm_write!(serial, "  Supported command classes: ");
            for i in 0..12 {
                uwrite!(serial, "{}", (csd.supported_command_classes >> (11 - i)) & 0x01).void_unwrap();
            }
            uwrite!(serial, "\n").void_unwrap();
            pm_write!(
                serial,
                "  Max data read block size:  {}\n",
                csd.max_read_block_len_bytes,
            );
            pm_write!(serial, "  Card capacity:             {} MiB\n", csd.capacity_mib);
        },
        Err(e) => {
            pm_write!(serial, "couldn't read CSD register: {}\n", e as u8);
            panic!("Aborting...");
        },
    }

    // let mut mbr: MbrSector = MbrSector::new();
    // let raw_mbr = unsafe { core::slice::from_raw_parts_mut((&mut mbr as *mut MbrSector) as *mut u8,
    // 512) }; if let Err(e) = sdcard.read_sectors(0, raw_mbr) {
    //     panic!("Could not read MBR");
    // }

    let mut raw_mbr = [0u8; 512];
    if let Err(e) = sdcard.read_sectors(0, &mut raw_mbr) {
        panic!("Could not read MBR");
    }

    // for i in 0..512 {
    //     pm_writeln!(serial, "data[{}] = {}", i, raw_mbr[i]).void_unwrap();
    // }
    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut led: Pin<Output, PB5> = unsafe { core::mem::MaybeUninit::uninit().assume_init() };
    let mut serial: Usart0<MHz16> = unsafe { core::mem::MaybeUninit::uninit().assume_init() };
    unsafe {
        let sp_high: u16 = *(0x5E as *const u16);
        let sp_low: u16 = *(0x5D as *const u16);
        uwriteln!(&mut serial, "Firmware panic!  SP = {} {}\r", sp_high, sp_low).void_unwrap();
    }

    if let Some(loc) = info.location() {
        uwriteln!(&mut serial, "  At {}:{}:{}\r", loc.file(), loc.line(), loc.column()).void_unwrap();
    }
    if let Some(message_args) = info.message() {
        if let Some(message) = message_args.as_str() {
            uwriteln!(&mut serial, "    {}\r", message).void_unwrap();
        }
    }

    loop {
        led.set_high();
        arduino_hal::delay_ms(100);
        led.set_low();
        arduino_hal::delay_ms(100);
    }
}
