mod cardinfo;
mod cmd;
mod crc;
mod debug;
mod init;
mod rwdata;

use atmega_hal::spi::{
    ChipSelectPin,
    Spi,
};
use avr_hal_generic::{
    prelude::*,
    spi,
};
use core::cell::RefCell;
use embedded_hal::spi::{
    FullDuplex,
    MODE_0,
};

#[derive(PartialEq, Eq)]
enum DataMode {
    Idle,
    Read,
    Write,
}

struct RwState {
    mode: DataMode,
    sector: u32,
}

pub enum SdVersion {
    One,
    Two { sdhc: bool },
}

pub struct SdCard<CSPIN: avr_hal_generic::port::PinOps> {
    pub version: SdVersion,

    spi: Spi,
    cs_pin: ChipSelectPin<CSPIN>,
    rw_state: RwState,

    millis: fn() -> u32,
}

pub enum SdCardError {
    NoResponse = 1,
    EraseReset,
    IllegalCommand,
    CRCError,
    EraseSequenceError,
    AddressError,
    ParameterError,
    RegisterError,
    ReadError,
    SDVersionOneUnsupported,
    CardCheckPatternMismatch,
    Timeout,
    Unknown,
}

impl<CSPIN: avr_hal_generic::port::PinOps> SdCard<CSPIN> {
    pub fn new(
        spi: Spi,
        cs_pin: ChipSelectPin<CSPIN>,
        millis: fn() -> u32,
    ) -> Result<RefCell<SdCard<CSPIN>>, SdCardError> {
        let mut sdcard = SdCard {
            version: SdVersion::Two { sdhc: false },

            spi,
            cs_pin,
            rw_state: RwState {
                mode: DataMode::Idle,
                sector: 0,
            },

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
