use super::{
    cmd::SdRegister,
    SdCard,
    SdCardError,
};

pub struct CardId {
    pub manufacturer_id: u8,
    pub oem_id: (char, char),
    pub product_name: [char; 5],
    pub product_revision: (u8, u8),
    pub product_serial_num: u32,
    pub manufacturing_date_year: u16,
    pub manufacturing_date_month: u8,
}

pub struct CardSpecificData {
    pub version: u8,
    pub tran_speed_mhz: u8,
    pub supported_command_classes: u16,
    pub max_read_block_len_bytes: u16,
    pub capacity_mib: u32,
}

impl<CSPIN: avr_hal_generic::port::PinOps> SdCard<CSPIN> {
    pub fn read_card_id(&mut self) -> Result<CardId, SdCardError> {
        let data = self.read_register(SdRegister::CID)?;
        Ok(CardId {
            manufacturer_id: data[0],
            oem_id: (data[1] as char, data[2] as char),
            product_name: [
                data[3] as char,
                data[4] as char,
                data[5] as char,
                data[6] as char,
                data[7] as char,
            ],
            product_revision: (data[8] & 0xf0, data[8] & 0x0f),
            product_serial_num: (data[9] as u32) << 24
                | (data[10] as u32) << 16
                | (data[11] as u32) << 8
                | (data[12] as u32),
            manufacturing_date_year: 2000 + ((data[13] << 4) as u16 | (data[14] >> 4) as u16),
            manufacturing_date_month: data[14] & 0x0f,
        })
    }

    pub fn read_card_specific_data(&mut self) -> Result<CardSpecificData, SdCardError> {
        let data = self.read_register(SdRegister::CSD)?;
        let version = (data[0] >> 6) + 1;
        if version != 2 {
            return Err(SdCardError::SDVersionOneUnsupported);
        }
        Ok(CardSpecificData {
            version,
            tran_speed_mhz: match data[3] {
                0x32 => 25,
                0x5a => 50,
                _ => 0,
            },
            supported_command_classes: ((data[4] as u16) << 4) | ((data[5] as u16) >> 4),
            max_read_block_len_bytes: match data[5] & 0x0f {
                0x09 => 512,
                _ => 0,
            },
            capacity_mib: ((((((data[7] & 0x3f) as u32) << 16 | (data[8] as u32) << 8 | (data[9] as u32)) + 1) as u64
                * 512000)
                >> 20) as u32,
        })
    }
}
