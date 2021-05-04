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
use ufmt::uwriteln;

const SD_CMD0_RETRY_COUNT: u8 = 10;
const R1_IDLE_STATE: u8 = 0x01;
const R1_ILLEGAL_COMMAND: u8 = 0x04;

enum SdVersion {
    One,
    Two,
}

pub struct SdCard<CSPIN: avr_hal_generic::port::PinOps> {
    spi: Spi,
    cs_pin: ChipSelectPin<CSPIN>,
    version: SdVersion,
}

enum SdCardError {
    NoResponse,
}

#[derive(Clone, Copy, PartialEq)]
enum SdCommand {
    GoIdleState = 0x0,
    SendIfCond = 0x8,
}

impl<CSPIN: avr_hal_generic::port::PinOps> SdCard<CSPIN> {
    pub fn new(spi: Spi, cs_pin: ChipSelectPin<CSPIN>, serial: &mut Usart0<MHz16>) -> SdCard<CSPIN> {
        let mut sdcard = SdCard {
            spi,
            cs_pin,
            version: SdVersion::Two,
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
            let response = sdcard.send_card_command(SdCommand::GoIdleState, 0x0, serial);
            match response {
                Ok(b) if b == R1_IDLE_STATE => {
                    uwriteln!(serial, "SD card in SPI mode").void_unwrap();
                    init = true
                },
                Ok(b) => uwriteln!(serial, "Sent CMD0, got {}", b).void_unwrap(),
                Err(_) => uwriteln!(serial, "Sent CMD0, got error").void_unwrap(),
            }
            i += 1;
        }

        // Check the SD version, older cards won't recognize this command
        // 0x1AA = 3.3V, check pattern of 0xAA; the SD card should return
        // the check pattern to ensure correct operation.
        let response = sdcard.send_card_command(SdCommand::SendIfCond, 0x1AA, serial);
        match response {
            Ok(b) => {
                uwriteln!(serial, "Sent CMD8, got {}", b).void_unwrap();
                if b & R1_ILLEGAL_COMMAND != 0 {
                    sdcard.version = SdVersion::One;
                } else {
                    // CMD8 has a 40 bit response, and the last 8 bits should match the
                    // check pattern.  We don't care about the other bits so just discard
                    // (the first 8 bits were the response value from `send_card_command`)
                    let mut check_pattern: u8 = 0;
                    for _ in 0..4 {
                        check_pattern = sdcard.transfer(0xff);
                        uwriteln!(serial, "check_pattern result = {}", check_pattern).void_unwrap();
                    }
                    if check_pattern != 0xAA {
                        panic!("SD card check pattern mismatch");
                    }
                }
            },
            Err(_) => uwriteln!(serial, "Sent CMD8, got error").void_unwrap(),
        }

        sdcard.cs_pin.set_high().void_unwrap();

        sdcard
    }

    fn transfer(&mut self, byte: u8) -> u8 {
        nb::block!(self.spi.send(byte)).void_unwrap();
        nb::block!(self.spi.read()).void_unwrap()
    }

    fn send_card_command(&mut self, cmd: SdCommand, arg: u32, serial: &mut Usart0<MHz16>) -> Result<u8, SdCardError> {
        // Wait for card to be ready
        let mut ready = 0;
        while cmd != SdCommand::GoIdleState && ready != 0xff {
            ready = self.transfer(0xff);
        }

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
        uwriteln!(serial, "crc is {}", crc).void_unwrap();

        // SD card needs at least 8 clock cycles to respond to a command
        self.transfer(0xff);

        // Poll for a response
        let mut res = Err(SdCardError::NoResponse);
        for _ in 0..10 {
            let response = self.transfer(0xff);

            // If the last bit is 1 the card hasn't responded yet
            if response & 0x80 == 0 {
                res = Ok(response);
                break;
            }
        }
        res
    }
}
