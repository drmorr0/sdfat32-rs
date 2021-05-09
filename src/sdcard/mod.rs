mod cardinfo;
mod cmd;
mod crc;
mod init;

use atmega_hal::{
    spi::{
        ChipSelectPin,
        Spi,
    },
    usart::Usart0,
};
use avr_hal_generic::{
    clock::MHz16,
    prelude::*,
    spi,
};
use embedded_hal::spi::{
    FullDuplex,
    MODE_0,
};
use ufmt::{
    derive::uDebug,
    uwriteln,
};

enum SdVersion {
    One,
    Two { sdhc: bool },
}

pub struct SdCard<CSPIN: avr_hal_generic::port::PinOps> {
    spi: Spi,
    cs_pin: ChipSelectPin<CSPIN>,
    millis: fn() -> u32,
    version: SdVersion,
}

#[derive(Debug, uDebug)]
pub enum SdCardError {
    NoResponse,
    EraseReset,
    IllegalCommand,
    CRCError,
    EraseSequenceError,
    AddressError,
    ParameterError,
    RegisterError,
    ReadError,
    Timeout,
    Unknown,
}

impl<CSPIN: avr_hal_generic::port::PinOps> SdCard<CSPIN> {
    pub fn new(
        spi: Spi,
        cs_pin: ChipSelectPin<CSPIN>,
        millis: fn() -> u32,
        serial: &mut Usart0<MHz16>,
    ) -> SdCard<CSPIN> {
        let mut sdcard = SdCard {
            spi,
            cs_pin,
            millis,
            version: SdVersion::Two { sdhc: false },
        };

        uwriteln!(serial, "----------------").void_unwrap();
        uwriteln!(serial, "Initializing SD card...").void_unwrap();
        let start_time_ms = (sdcard.millis)();
        // Need to hold CS and MOSI high for at least 74 clock cycles;
        // each transfer takes 8 clock cycles so repeating for 10 times is sufficient
        sdcard.cs_pin.set_high().void_unwrap();
        for _ in 0..10 {
            sdcard.transfer(0xff);
        }

        sdcard.select();
        sdcard.init_spi(serial);
        sdcard.check_sd_version(serial);
        sdcard.enable_crc(serial);
        sdcard.check_and_enable_sdhc(serial);
        sdcard.unselect();

        // Once initialization is complete we can bump the SPI speed up to max
        // (cards support up to 25MHz, OscfOver2 = 8MHz)
        nb::block!(sdcard.spi.reconfigure(spi::Settings {
            data_order: spi::DataOrder::MostSignificantFirst,
            clock: spi::SerialClockRate::OscfOver2,
            mode: MODE_0,
        }))
        .void_unwrap();

        uwriteln!(
            serial,
            "Initialization complete ({} ms)",
            (sdcard.millis)() - start_time_ms,
        )
        .void_unwrap();
        uwriteln!(serial, "----------------").void_unwrap();

        sdcard
    }

    #[inline(always)]
    fn select(&mut self) {
        // Set CS to low to indicate we're talking
        self.cs_pin.set_low().void_unwrap();
    }

    #[inline(always)]
    fn unselect(&mut self) {
        // Set CS to high when we're all finished
        self.cs_pin.set_high().void_unwrap();
    }

    fn transfer(&mut self, byte: u8) -> u8 {
        nb::block!(self.spi.send(byte)).void_unwrap();
        nb::block!(self.spi.read()).void_unwrap()
    }
}
