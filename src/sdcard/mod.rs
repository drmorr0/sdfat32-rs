mod crc;

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
};
use crc::CRC7;
use embedded_hal::spi::FullDuplex;
use ufmt::{
    derive::uDebug,
    uwriteln,
};

const SD_CMD0_RETRY_COUNT: u8 = 10;
const SD_INIT_TIMEOUT_MS: u32 = 2000;

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
enum SdCardError {
    NoResponse,
    EraseReset,
    IllegalCommand,
    CRCError,
    EraseSequenceError,
    AddressError,
    ParameterError,
    Timeout,
    Unknown,
}

enum SdCommand {
    GoIdleState = 0,
    AppCommand = 55,
    SetCRC = 59,
}

#[derive(Clone, Copy)]
enum SdAppCommand {
    SendOpCondition = 41,
}

enum SdCommandWide {
    SendIfCond = 8,
    ReadOCR = 58,
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

        // Need to hold CS and MOSI high for at least 74 clock cycles;
        // each transfer takes 8 clock cycles so repeating for 10 times is sufficient
        sdcard.cs_pin.set_high().void_unwrap();
        for _ in 0..10 {
            sdcard.transfer(0xff);
        }

        // Set CS to low to indicate we're talking
        sdcard.cs_pin.set_low().void_unwrap();

        // Put the card in SPI mode
        let (mut i, mut init) = (0, false);
        while !init {
            if i >= SD_CMD0_RETRY_COUNT {
                panic!("Could not initialize SD card");
            }
            match sdcard.send_card_command(SdCommand::GoIdleState, 0x0) {
                Ok(()) => {
                    uwriteln!(serial, "SD card in SPI mode").void_unwrap();
                    init = true
                },
                Err(e) => uwriteln!(serial, "Error sending CMD0: {:?}", e).void_unwrap(),
            }
            i += 1;
        }

        // Check the SD version, older cards won't recognize this command
        // 0x1AA = 3.3V, check pattern of 0xAA; the SD card should return
        // the check pattern to ensure correct operation.
        match sdcard.send_card_command_wide(SdCommandWide::SendIfCond, 0x1AA) {
            Ok(data) => {
                // CMD8 has a 40 bit response, and the last 8 bits should match the
                // check pattern.
                if data[3] != 0xAA {
                    panic!("SD card check pattern mismatch");
                } else {
                    uwriteln!(serial, "SD card version 2 detected").void_unwrap();
                }
            },
            Err(SdCardError::IllegalCommand) => {
                sdcard.version = SdVersion::One;
                panic!("SD card version one currently unsupported");
            },
            Err(e) => panic!("Error sending CMD8: {:?}", e),
        }

        // Enable the CRC for all commands/data
        match sdcard.send_card_command(SdCommand::SetCRC, 1) {
            Ok(()) => uwriteln!(serial, "CRC enabled").void_unwrap(),
            Err(e) => panic!("Error sending CMD59: {:?}", e),
        }

        // Send "host supports SDHC" if version 2
        let acmd41_arg = match sdcard.version {
            SdVersion::Two { sdhc: _ } => 0x40000000,
            SdVersion::One => 0,
        };
        match sdcard.send_card_app_command(SdAppCommand::SendOpCondition, acmd41_arg) {
            Ok(()) => uwriteln!(serial, "SD card initialization complete").void_unwrap(),
            Err(e) => panic!("Error sending ACMD41: {:?}", e),
        }

        // Check if card supports SDHC
        match sdcard.send_card_command_wide(SdCommandWide::ReadOCR, 0) {
            Ok(data) => {
                if data[0] & 0xC0 == 0xC0 {
                    uwriteln!(serial, "SD card supports SDHC").void_unwrap();
                    sdcard.version = SdVersion::Two { sdhc: true };
                }
            },
            Err(e) => panic!("Error sending CMD58: {:?}", e),
        }

        sdcard.cs_pin.set_high().void_unwrap();

        sdcard
    }

    fn transfer(&mut self, byte: u8) -> u8 {
        nb::block!(self.spi.send(byte)).void_unwrap();
        nb::block!(self.spi.read()).void_unwrap()
    }

    fn send_card_app_command(&mut self, cmd: SdAppCommand, arg: u32) -> Result<(), SdCardError> {
        let start_time_ms = (self.millis)();
        while (self.millis)() <= start_time_ms + SD_INIT_TIMEOUT_MS {
            self.send_card_command_helper(SdCommand::AppCommand as u8, 0)?;
            match self.send_card_command_helper(cmd as u8, arg) {
                Ok(b) if b == 0x0 => return Ok(()),
                Ok(_) => continue,
                Err(e) => return Err(e),
            }
        }
        Err(SdCardError::Timeout)
    }

    fn send_card_command(&mut self, cmd: SdCommand, arg: u32) -> Result<(), SdCardError> {
        match self.send_card_command_helper(cmd as u8, arg) {
            Ok(0x01) => Ok(()),
            Ok(b) if b & 0x02 != 0 => Err(SdCardError::EraseReset),
            Ok(b) if b & 0x04 != 0 => Err(SdCardError::IllegalCommand),
            Ok(b) if b & 0x08 != 0 => Err(SdCardError::CRCError),
            Ok(b) if b & 0x08 != 0 => Err(SdCardError::EraseSequenceError),
            Ok(b) if b & 0x10 != 0 => Err(SdCardError::AddressError),
            Ok(b) if b & 0x20 != 0 => Err(SdCardError::ParameterError),
            Ok(_) => Err(SdCardError::Unknown),
            Err(e) => Err(e),
        }
    }

    fn send_card_command_wide(&mut self, cmd: SdCommandWide, arg: u32) -> Result<[u8; 4], SdCardError> {
        let mut response = [0, 0, 0, 0];
        self.send_card_command_helper(cmd as u8, arg)?;
        for i in 0..4 {
            response[i] = self.transfer(0xff);
        }
        Ok(response)
    }

    fn send_card_command_helper(&mut self, cmd: u8, arg: u32) -> Result<u8, SdCardError> {
        // Wait for card to be ready
        while cmd != (SdCommand::GoIdleState as u8) && self.transfer(0xff) != 0xff {}

        // Command format is 01CCCCCCAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARRRRRRR1
        // where C is the 6-bit command, A is the 32-bit argument, and R is the 7-bit CRC
        let data = [
            0x40 | cmd as u8,
            (arg >> 24) as u8,
            (arg >> 16) as u8,
            (arg >> 8) as u8,
            arg as u8,
        ];
        let crc = CRC7(data);
        for byte in data.iter() {
            self.transfer(*byte);
        }
        self.transfer(crc);

        // SD card needs at least 8 clock cycles to respond to a command
        self.transfer(0xff);

        // Poll for a response
        for _ in 0..10 {
            let response = self.transfer(0xff);

            // If the last bit is 1 the card hasn't responded yet
            if response & 0x80 == 0 {
                return Ok(response);
            }
        }
        Err(SdCardError::NoResponse)
    }
}
