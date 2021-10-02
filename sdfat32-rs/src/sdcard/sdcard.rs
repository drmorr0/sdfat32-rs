use super::SdCardError;
use atmega_hal::spi::{
    ChipSelectPin,
    Spi,
};
use avr_hal_generic::{
    port::PinOps,
    prelude::*,
    spi,
};
use core::cell::RefCell;
use embedded_hal::spi::{
    FullDuplex,
    MODE_0,
};

pub enum SdVersion {
    One,
    Two { sdhc: bool },
}

pub struct SdCard<CSPIN: PinOps> {
    pub version: SdVersion,
    pub(crate) millis: fn() -> u32,
    spi: Spi,
    cs_pin: ChipSelectPin<CSPIN>,
}

impl<CSPIN: PinOps> SdCard<CSPIN> {
    pub fn new(
        spi: Spi,
        cs_pin: ChipSelectPin<CSPIN>,
        millis: fn() -> u32,
    ) -> Result<RefCell<SdCard<CSPIN>>, SdCardError> {
        let mut sdcard = SdCard {
            version: SdVersion::Two { sdhc: false },
            spi,
            cs_pin,
            millis,
        };

        // Need to hold CS and MOSI high for at least 74 clock cycles;
        // each transfer takes 8 clock cycles so repeating for 10 times is sufficient
        sdcard.cs_pin.set_high().void_unwrap();
        for _ in 0..10 {
            sdcard.transfer(0xff);
        }

        sdcard.select();
        sdcard.init_spi()?;
        sdcard.check_sd_version()?;
        sdcard.enable_crc()?;
        sdcard.check_and_enable_sdhc()?;
        sdcard.unselect();

        // Once initialization is complete we can bump the SPI speed up to max
        // (cards support up to 25MHz, OscfOver2 = 8MHz)
        nb::block!(sdcard.spi.reconfigure(spi::Settings {
            data_order: spi::DataOrder::MostSignificantFirst,
            clock: spi::SerialClockRate::OscfOver2,
            mode: MODE_0,
        }))
        .void_unwrap();

        Ok(RefCell::new(sdcard))
    }

    #[inline(always)]
    pub(crate) fn select(&mut self) {
        // Set CS to low to indicate we're talking
        self.cs_pin.set_low().void_unwrap();
    }

    #[inline(always)]
    pub(crate) fn unselect(&mut self) {
        // Set CS to high when we're all finished
        self.cs_pin.set_high().void_unwrap();
    }

    pub(crate) fn transfer(&mut self, byte: u8) -> u8 {
        nb::block!(self.spi.send(byte)).void_unwrap();
        nb::block!(self.spi.read()).void_unwrap()
    }
}

pub type SdCardRef<'s, CSPIN> = &'s RefCell<SdCard<CSPIN>>;
