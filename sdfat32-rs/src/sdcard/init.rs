use super::{
    cmd::{
        SdAppCommand,
        SdCommand,
        SdCommandWide,
    },
    constants::*,
    SdCard,
    SdCardError,
    SdVersion,
};


impl<CSPIN: avr_hal_generic::port::PinOps> SdCard<CSPIN> {
    pub(crate) fn init_spi(&mut self) -> Result<(), SdCardError> {
        let (mut i, mut init) = (0, false);
        while !init {
            if i >= SD_CMD0_RETRY_COUNT {
                return Err(SdCardError::Unknown);
            }
            self.send_card_command(SdCommand::GoIdleState, 0x0)?;
            init = true;
            i += 1;
        }
        Ok(())
    }

    pub(crate) fn check_and_enable_sdhc(&mut self) -> Result<(), SdCardError> {
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
                Err(e) => return Err(e),
            }
        }

        if !init_done {
            return Err(SdCardError::Timeout);
        }

        // Check if card supports SDHC
        match self.send_card_command_wide(SdCommandWide::ReadOCR, 0) {
            Ok(data) => {
                if data[0] & 0xC0 == 0xC0 {
                    self.version = SdVersion::Two { sdhc: true };
                }
            },
            Err(e) => return Err(e),
        }

        Ok(())
    }

    pub(crate) fn check_sd_version(&mut self) -> Result<(), SdCardError> {
        // Older (SDv1) cards won't recognize this command.  The argument
        // 0x1AA means 3.3V and check pattern of 0xAA; the SD card should return
        // the check pattern to ensure correct operation.
        match self.send_card_command_wide(SdCommandWide::SendIfCond, 0x1AA) {
            Ok(data) => {
                // CMD8 has a 40 bit response, and the last 8 bits should match the
                // check pattern.
                if data[3] != 0xAA {
                    return Err(SdCardError::CardCheckPatternMismatch);
                }
            },
            Err(SdCardError::IllegalCommand) => {
                self.version = SdVersion::One;
                return Err(SdCardError::SDVersionOneUnsupported);
            },
            Err(e) => return Err(e),
        }
        Ok(())
    }

    pub(crate) fn enable_crc(&mut self) -> Result<(), SdCardError> {
        // By default in SPI mode only CMD0 and CMD8 are CRC-checked;
        // this command will enable it for all commands
        self.send_card_command(SdCommand::SetCRC, 1)?;
        Ok(())
    }
}
