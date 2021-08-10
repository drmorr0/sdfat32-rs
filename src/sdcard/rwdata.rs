use super::{
    cmd::SdCommand,
    DataMode,
    SdCard,
    SdCardError,
};

const READ_BLOCK_SIZE_V2: usize = 512;

impl<CSPIN: avr_hal_generic::port::PinOps> SdCard<CSPIN> {
    pub fn read_sectors(&mut self, start_sector: u32, data: &mut [u8]) -> Result<(), SdCardError> {
        if self.rw_state.mode != DataMode::Read || self.rw_state.sector != start_sector {
            self.select();
            self.send_card_command(SdCommand::ReadMultipleBlocks, start_sector)?;
            self.rw_state.mode = DataMode::Read;
            self.rw_state.sector = start_sector;
        }

        for i in (0..data.len()).step_by(READ_BLOCK_SIZE_V2) {
            self.read_data(&mut data[i..i + READ_BLOCK_SIZE_V2])?;
            self.rw_state.sector += 1;
        }
        self.send_card_command(SdCommand::ReadStop, 0)?;
        self.unselect();
        Ok(())
    }
}
