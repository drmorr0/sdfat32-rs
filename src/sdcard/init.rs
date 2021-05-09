use super::{
    cmd::{
        SdAppCommand,
        SdCommand,
        SdCommandWide,
    },
    SdCard,
    SdCardError,
    SdVersion,
};
use atmega_hal::usart::Usart0;
use avr_hal_generic::{
    clock::MHz16,
    prelude::*,
};
use ufmt::uwriteln;

const SD_CMD0_RETRY_COUNT: u8 = 10;
const SD_INIT_TIMEOUT_MS: u32 = 2000;

impl<CSPIN: avr_hal_generic::port::PinOps> SdCard<CSPIN> {
    pub(crate) fn init_spi(&mut self, serial: &mut Usart0<MHz16>) {
        let (mut i, mut init) = (0, false);
        while !init {
            if i >= SD_CMD0_RETRY_COUNT {
                panic!("Could not initialize SD card");
            }
            match self.send_card_command(SdCommand::GoIdleState, 0x0) {
                Ok(()) => {
                    uwriteln!(serial, "SD card in SPI mode").void_unwrap();
                    init = true
                },
                Err(e) => uwriteln!(serial, "Error sending CMD0: {:?}", e).void_unwrap(),
            }
            i += 1;
        }
    }

    pub(crate) fn check_sd_version(&mut self, serial: &mut Usart0<MHz16>) {
        // Older (SDv1) cards won't recognize this command.  The argument
        // 0x1AA means 3.3V and check pattern of 0xAA; the SD card should return
        // the check pattern to ensure correct operation.
        match self.send_card_command_wide(SdCommandWide::SendIfCond, 0x1AA) {
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
                self.version = SdVersion::One;
                panic!("SD card version one currently unsupported");
            },
            Err(e) => panic!("Error sending CMD8: {:?}", e),
        }
    }

    pub(crate) fn enable_crc(&mut self, serial: &mut Usart0<MHz16>) {
        // By default in SPI mode only CMD0 and CMD8 are CRC-checked;
        // this command will enable it for all commands
        match self.send_card_command(SdCommand::SetCRC, 1) {
            Ok(()) => uwriteln!(serial, "CRC enabled").void_unwrap(),
            Err(e) => panic!("Error sending CMD59: {:?}", e),
        }
    }

    pub(crate) fn check_and_enable_sdhc(&mut self, serial: &mut Usart0<MHz16>) {
        // Send "host supports SDHC" if version 2
        let acmd41_arg = match self.version {
            SdVersion::Two { sdhc: _ } => 0x40000000,
            SdVersion::One => 0,
        };

        let (start_time_ms, mut init_done) = ((self.millis)(), false);
        while (self.millis)() <= start_time_ms + SD_INIT_TIMEOUT_MS && !init_done {
            match self.send_card_app_command(SdAppCommand::SendOpCondition, acmd41_arg) {
                Ok(b) if b == 0x0 => init_done = true,
                Ok(_) => continue,
                Err(e) => panic!("Error sending ACMD41: {:?}", e),
            }
        }

        if !init_done {
            panic!("Error initializing SD card: Timeout");
        }

        // Check if card supports SDHC
        match self.send_card_command_wide(SdCommandWide::ReadOCR, 0) {
            Ok(data) => {
                if data[0] & 0xC0 == 0xC0 {
                    uwriteln!(serial, "SD card supports SDHC").void_unwrap();
                    self.version = SdVersion::Two { sdhc: true };
                }
            },
            Err(e) => panic!("Error sending CMD58: {:?}", e),
        }
    }
}
