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

    let sdcard = SdCard::new(spi, cs, millis, &mut serial);
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
