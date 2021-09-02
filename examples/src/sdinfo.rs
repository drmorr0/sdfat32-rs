#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(abi_avr_interrupt)]
#![feature(panic_info_message)]
#![allow(unreachable_code)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(deprecated)] // llvm_asm!

mod strings;

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
use avr_progmem_str::{
    pm_write,
    progmem_str,
};
use embedded_hal::spi::MODE_0;
use sdfat32_rs::{
    fat32,
    sdcard::{
        SdCard,
        SdVersion,
    },
};
use ufmt::{
    uwrite,
    uwriteln,
};

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
            pm_write!(serial, "  Manufacturer ID:  ");
            strings::mid_write(&mut serial, cid.manufacturer_id);
            serial.write_char('\n').unwrap();
            pm_write!(serial, "  OEM ID:           {}{}\n", cid.oem_id.0, cid.oem_id.1);
            pm_write!(serial, "  Product name:     ");
            for i in 0..5 {
                serial.write_char(cid.product_name[i]).unwrap();
            }
            serial.write_char('\n').unwrap();
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
            serial.write_char('\n').unwrap();
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

    pm_write!(serial, "\nMaster Boot Record:\n");
    let mut mbr = fat32::Mbr::new();
    match mbr.read(&mut sdcard) {
        Ok(()) => {
            for (i, partition) in mbr.partitions.iter().enumerate() {
                pm_write!(
                    serial,
                    "  Partition {}: is_boot = {}; partition_type = ",
                    i,
                    partition.boot,
                );
                strings::ptype_write(&mut serial, partition.ptype);
                pm_write!(
                    serial,
                    "; begin_chs = {}/{}/{}; end_chs = {}/{}/{}; start_sector = {}, length = {},\n",
                    partition.begin_chs[0],
                    partition.begin_chs[1],
                    partition.begin_chs[2],
                    partition.end_chs[0],
                    partition.end_chs[1],
                    partition.end_chs[2],
                    partition.start_sector,
                    partition.total_sectors,
                );
            }
        },
        Err(e) => {
            pm_write!(serial, "Couldn't read MBR: {}\n", e as u8);
            panic!("Aborting...");
        },
    }

    pm_write!(serial, "\nPartition 0:\n");
    match fat32::Partition::read(&mut sdcard, &mbr.partitions[0]) {
        Ok(part) => {
            pm_write!(serial, "  sectors per cluster: {}\n", part.sectors_per_cluster);
            pm_write!(serial, "  cluster count:       {}\n", part.cluster_count);
            pm_write!(serial, "  fat start sector:    {}\n", part.fat_start_sector);
        },
        Err(e) => {
            pm_write!(serial, "Couldn't read partition: {}\n", e as u8);
            panic!("Aborting...");
        },
    }

    loop {}
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let mut led: Pin<Output, PB5> = unsafe { core::mem::MaybeUninit::uninit().assume_init() };
    let mut serial: Usart0<MHz16> = unsafe { core::mem::MaybeUninit::uninit().assume_init() };
    let _ = serial.write_str("panic!\r"); // Ignore failures because we're already panicking...

    loop {
        led.set_high();
        arduino_hal::delay_ms(100);
        led.set_low();
        arduino_hal::delay_ms(100);
    }
}
