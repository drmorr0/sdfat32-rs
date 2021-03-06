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
    },
    sdcard::SdCard,
};

const FILENAME: [u8; 34] = [
    '/' as u8, 'a' as u8, ' ' as u8, 'r' as u8, 'e' as u8, 'a' as u8, 'l' as u8, 'l' as u8, 'y' as u8, ' ' as u8,
    'l' as u8, 'o' as u8, 'n' as u8, 'g' as u8, '+' as u8, 'f' as u8, 'i' as u8, 'l' as u8, 'n' as u8, 'a' as u8,
    'm' as u8, 'e' as u8, '.' as u8, 't' as u8, 'x' as u8, 't' as u8, 'l' as u8, 'o' as u8, 'l' as u8, 0, 0, 0, 0, 0,
];


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
                Ok(mut vol) => {
                    pm_write!(serial, "Trying to read ").void_unwrap();
                    for c in FILENAME {
                        if c != 0x0 {
                            serial.write_char(c as char).void_unwrap();
                        }
                    }
                    serial.write_char('\n').void_unwrap();
                    match vol.open_by_name(&sdcard, &FILENAME, O_RDONLY) {
                        Ok(mut file) => {
                            let mut buffer = [0u8; 40];
                            match vol.read(&sdcard, &mut file, &mut buffer) {
                                Ok(n) => {
                                    for i in 0..n {
                                        serial.write_char(buffer[i] as char).void_unwrap();
                                    }
                                },
                                Err(e) => {
                                    pm_write!(serial, "Couldn't read file contents: {}\n", e as u8).void_unwrap();
                                    panic!("");
                                },
                            }
                        },
                        Err(e) => {
                            pm_write!(serial, "Couldn't read file: {}\n", e as u8).void_unwrap();
                            panic!("");
                        },
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
