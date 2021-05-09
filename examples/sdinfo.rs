#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(abi_avr_interrupt)]
#![feature(panic_info_message)]

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
use embedded_hal::spi::MODE_0;
use sdfat_rs::sdcard::SdCard;
use ufmt::uwriteln;

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
    let dp = arduino_hal::Peripherals::take().unwrap();
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

    let mut sdcard = SdCard::new(spi, cs, millis, &mut serial);

    uwriteln!(serial, "CardInfo: ").void_unwrap();
    match sdcard.read_card_id() {
        Ok(cid) => {
            uwriteln!(
                serial,
                "  Manufacturer ID:  {} ({})",
                mid_lookup(cid.manufacturer_id),
                cid.manufacturer_id,
            )
            .void_unwrap();
            uwriteln!(serial, "  OEM ID:           {}{}", cid.oem_id.0, cid.oem_id.1).void_unwrap();
            uwriteln!(
                serial,
                "  Product Name:     {}{}{}{}{}",
                cid.product_name.0,
                cid.product_name.1,
                cid.product_name.2,
                cid.product_name.3,
                cid.product_name.4
            )
            .void_unwrap();
            uwriteln!(
                serial,
                "  Product revision: {}.{}",
                cid.product_revision.0,
                cid.product_revision.1,
            )
            .void_unwrap();
            uwriteln!(
                serial,
                "  Serial number:    {}{}",
                (cid.product_serial_num >> 16),
                cid.product_serial_num
            )
            .void_unwrap();
            uwriteln!(
                serial,
                "  Manufacture date: {}-{}",
                cid.manufacturing_date_year,
                cid.manufacturing_date_month,
            )
            .void_unwrap();
        },
        Err(e) => {
            uwriteln!(serial, "couldn't read CID register: {:?}", e).void_unwrap();
            panic!("Aborting...");
        },
    }

    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut led: Pin<Output, PB5> = unsafe { core::mem::MaybeUninit::uninit().assume_init() };

    let mut serial: Usart0<MHz16> = unsafe { core::mem::MaybeUninit::uninit().assume_init() };
    uwriteln!(&mut serial, "Firmware panic!\r").void_unwrap();

    if let Some(loc) = info.location() {
        uwriteln!(&mut serial, "  At {}:{}:{}\r", loc.file(), loc.line(), loc.column(),).void_unwrap();
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
