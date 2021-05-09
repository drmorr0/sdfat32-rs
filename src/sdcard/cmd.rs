use super::{
    crc::CRC7,
    SdCard,
    SdCardError,
};

#[derive(Clone, Copy)]
pub(crate) enum SdCommand {
    GoIdleState = 0,
    AppCommand = 55,
    SetCRC = 59,
}

#[derive(Clone, Copy)]
pub(crate) enum SdAppCommand {
    SendOpCondition = 41,
}

#[derive(Clone, Copy)]
pub(crate) enum SdCommandWide {
    SendIfCond = 8,
    ReadOCR = 58,
}

#[derive(Clone, Copy)]
pub enum SdRegister {
    CSD = 9,
    CID = 10,
}

const SD_READ_TIMEOUT_MS: u32 = 300;
const DATA_START_SECTOR: u8 = 0xfe;

impl<CSPIN: avr_hal_generic::port::PinOps> SdCard<CSPIN> {
    pub(crate) fn send_card_app_command(&mut self, cmd: SdAppCommand, arg: u32) -> Result<u8, SdCardError> {
        // Application-specific commands have to be preceded by CMD55 or they will error
        self.send_card_command_helper(SdCommand::AppCommand as u8, 0)?;
        self.send_card_command_helper(cmd as u8, arg)
    }

    pub(crate) fn send_card_command(&mut self, cmd: SdCommand, arg: u32) -> Result<(), SdCardError> {
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

    pub(crate) fn send_card_command_wide(&mut self, cmd: SdCommandWide, arg: u32) -> Result<[u8; 4], SdCardError> {
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

    pub fn read_register(&mut self, reg: SdRegister) -> Result<[u8; 16], SdCardError> {
        self.select();
        match self.send_card_command_helper(reg as u8, 0) {
            Ok(b) if b == 0 => (),
            Ok(_) => return Err(SdCardError::RegisterError),
            Err(e) => return Err(e),
        }

        let reg = self.read_data::<16>();
        self.unselect();
        reg
    }

    fn read_data<const N: usize>(&mut self) -> Result<[u8; N], SdCardError> {
        let mut data = [0; N];
        let start_time_ms = (self.millis)();

        let mut res = self.transfer(0xff);
        while res == 0xff {
            if (self.millis)() >= start_time_ms + SD_READ_TIMEOUT_MS {
                return Err(SdCardError::Timeout);
            }
            res = self.transfer(0xff);
        }

        if res != DATA_START_SECTOR {
            return Err(SdCardError::ReadError);
        }

        for i in 0..N {
            data[i] = self.transfer(0xff);
        }

        let _crc: u16 = ((self.transfer(0xff) as u16) << 8) | (self.transfer(0xff) as u16);

        Ok(data)
    }
}
