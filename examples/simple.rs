#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(abi_avr_interrupt)]

use arduino_hal::spi;
use atmega_hal::port::PB5;
use avr_hal_generic::port::{
    mode::Output,
    Pin,
};
use embedded_hal::spi::MODE_0;
use sdfat_rs::sdcard::SdCard;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let (spi, mut cs) = arduino_hal::Spi::new(
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

    let sdcard = SdCard::new(spi, cs, &mut serial);
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let mut led: Pin<Output, PB5> = unsafe { core::mem::MaybeUninit::uninit().assume_init() };

    // let mut serial: arduino_hal::Serial<Floating> = unsafe {
    // core::mem::MaybeUninit::uninit().assume_init() }; uwriteln!(&mut serial, "Firmware
    // panic!\r").void_unwrap();

    // if let Some(loc) = info.location() {
    //     ufmt::uwriteln!(&mut serial, "  At {}:{}:{}\r", loc.file(), loc.line(),
    // loc.column(),).void_unwrap(); }
    // if let Some(message_args) = info.message() {
    //     if let Some(message) = message_args.as_str() {
    //         ufmt::uwriteln!(&mut serial, "    {}\r", message).void_unwrap();
    //     }
    // }

    loop {
        led.set_high();
        arduino_hal::delay_ms(100);
        led.set_low();
        arduino_hal::delay_ms(100);
    }
}
