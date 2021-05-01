use atmega_hal::spi::ChipSelectPin;
use atmega_hal::spi::Spi;
use atmega_hal::usart::Usart0;
use avr_hal_generic::clock::MHz16;
use avr_hal_generic::prelude::*;
use embedded_hal::spi::FullDuplex;
use ufmt::uwriteln;

pub struct SdCard<CSPIN: avr_hal_generic::port::PinOps> {
    spi: Spi,
    cs_pin: ChipSelectPin<CSPIN>,
}

#[derive(Debug)]
enum SdCardError {
    Error,
}

#[derive(Clone, Copy)]
enum SdCommand {
    GoIdleState = 0x0,
    SendIfCond = 0x08,
}

impl<CSPIN: avr_hal_generic::port::PinOps> SdCard<CSPIN> {
    pub fn new(
        spi: Spi,
        cs_pin: ChipSelectPin<CSPIN>,
        serial: &mut Usart0<MHz16>,
    ) -> SdCard<CSPIN> {
        let mut sdcard = SdCard { spi, cs_pin };

        sdcard.cs_pin.set_high().void_unwrap();
        for _ in 0..10 {
            nb::block!(sdcard.spi.send(0xff)).void_unwrap();
        }
        sdcard.cs_pin.set_low().void_unwrap();

        loop {
            let response = sdcard.send_card_command(SdCommand::GoIdleState, 0x0, serial);
            match response {
                Ok(b) => { uwriteln!(serial, "Sent CMD0, got {}", b).void_unwrap(); break }
                Err(_) => uwriteln!(serial, "Sent CMD0, got error").void_unwrap(),
            }
        }
        sdcard.send_card_command(SdCommand::SendIfCond, 0x1AA, serial);
        uwriteln!(serial, "Sent CMD8");
        let mut status: u8 = 0;
        for _ in 0..4 {
            status = nb::block!(sdcard.spi.read()).void_unwrap();
        }
        uwriteln!(serial, "The card responded {}", status);

        sdcard
    }

    fn send_card_command(
        &mut self,
        cmd: SdCommand,
        arg: u32,
        serial: &mut Usart0<MHz16>,
    ) -> Result<u8, SdCardError> {
        self.cs_pin.set_low().void_unwrap();
        nb::block!(self.spi.send((0x40 | cmd as u8) as u8)).void_unwrap();
        nb::block!(self.spi.send((arg >> 24) as u8)).void_unwrap();
        nb::block!(self.spi.send((arg >> 16) as u8)).void_unwrap();
        nb::block!(self.spi.send((arg >> 8) as u8)).void_unwrap();
        nb::block!(self.spi.send(arg as u8)).void_unwrap();
        nb::block!(self.spi.send(0x95)).void_unwrap(); // TODO, only correct for CMD0

        nb::block!(self.spi.send(0xff)).void_unwrap();
        nb::block!(self.spi.read()).void_unwrap();

        for _ in 0..10 {
            nb::block!(self.spi.send(0xff)).void_unwrap();
            let byte = nb::block!(self.spi.read()).void_unwrap();
            uwriteln!(serial, "The card responded {}", byte);
            if byte & 0x80 == 0 {
                self.cs_pin.set_high().void_unwrap();
                return Ok(byte);
            }
        }
        self.cs_pin.set_high().void_unwrap();
        Err(SdCardError::Error)
    }
}
