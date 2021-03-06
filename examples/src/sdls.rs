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
    fat32::{
        self,
        constants::O_RDONLY,
        DirEntry,
    },
    sdcard::SdCard,
};
use ufmt::uwrite;

const RECURSION_DEPTH: u16 = 1;


fn print_entry(entry: &DirEntry, depth: u16, serial: &mut Usart0<MHz16>) {
    if entry.is_self_or_parent() {
        return;
    }
    for _ in 0..depth {
        serial.write_char(' ').void_unwrap();
    }

    match entry {
        DirEntry::Long(lfn, ..) => {
            for i in 0..13 {
                let c = lfn.get_char(i);
                if c == 0x0 || c == 0xff {
                    break;
                } else {
                    serial.write_char(c as char).void_unwrap();
                }
            }
        },
        DirEntry::Short(sfn, had_lfn) => {
            if !had_lfn {
                for c in sfn.name() {
                    serial.write_char(*c as char).void_unwrap();
                }
            }
            if sfn.is_directory() {
                serial.write_char('D').void_unwrap();
            } else {
                serial.write_char('F').void_unwrap();
            }
            uwrite!(serial, "    {}\n", sfn.size()).void_unwrap();
        },
    }
}


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

    match fat32::Mbr::read_part_info(&sdcard) {
        Ok(part_info) => {
            match fat32::Volume::open_volume(&sdcard, 0, &part_info[0]) {
                Ok(vol) => {
                    pm_write!(serial, "volume opened!  Contents:\n").void_unwrap();
                    let mut root = vol.open_root(O_RDONLY);
                    if let Err(e) = vol.ls(&sdcard, &mut root, false, 0, RECURSION_DEPTH, &mut serial, print_entry) {
                        pm_write!(serial, "Couldn't read directory: {}\n", e as u8).void_unwrap();
                        panic!("");
                    }
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
