#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(abi_avr_interrupt)]
#![feature(panic_info_message)]
#![allow(deprecated)] // llvm_asm!

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
use ufmt::uwrite;


#[arduino_hal::entry]
fn main() -> ! {
    let dp = match arduino_hal::Peripherals::take() {
        Some(p) => p,
        None => panic!(""),
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

    let sdcard = match SdCard::new(spi, cs, millis) {
        Ok(s) => s,
        Err(e) => {
            pm_write!(serial, "SdCard initialization failed with error code {}\n", e as u8).void_unwrap();
            panic!("")
        },
    };

    pm_write!(serial, "\nCardInfo:\n").void_unwrap();
    let version_str = match sdcard.borrow().version {
        SdVersion::One => "1.0",
        SdVersion::Two { sdhc: true } => "2.0 (SDHC)",
        SdVersion::Two { sdhc: false } => "2.0",
    };
    pm_write!(serial, "  SD Card Version:  {}\n", version_str).void_unwrap();

    match sdcard.borrow_mut().read_card_id() {
        Ok(cid) => uwrite!(serial, "{:?}", cid).void_unwrap(),
        Err(e) => {
            pm_write!(serial, "couldn't read CID register: {}\n", e as u8).void_unwrap();
            panic!("");
        },
    }
    pm_write!(serial, "\nCard-specific data:\n").void_unwrap();
    match sdcard.borrow_mut().read_card_specific_data() {
        Ok(csd) => uwrite!(serial, "{:?}", csd).void_unwrap(),
        Err(e) => {
            pm_write!(serial, "couldn't read CSD register: {}\n", e as u8).void_unwrap();
            panic!("");
        },
    }

    pm_write!(serial, "\nMaster Boot Record:\n").void_unwrap();
    match fat32::Mbr::read_part_info(&sdcard) {
        Ok(part_info) => {
            for (i, part_info) in part_info.iter().enumerate() {
                pm_write!(serial, "  Partition {}", i).void_unwrap();
                uwrite!(serial, "{:?}", part_info).void_unwrap();
            }

            pm_write!(serial, "\nPartition 0:\n").void_unwrap();
            match fat32::Volume::open_volume(&sdcard, 0, &part_info[0]) {
                Ok(vol) => {
                    uwrite!(serial, "{:?}", vol.partition).void_unwrap();
                },
                Err(e) => {
                    pm_write!(serial, "Couldn't read volume: {}\n", e as u8).void_unwrap();
                    panic!("");
                },
            };
        },
        Err(e) => {
            pm_write!(serial, "Couldn't read MBR: {}\n", e as u8).unwrap();
            panic!("");
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
