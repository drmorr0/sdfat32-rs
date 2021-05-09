use super::{
    cmd::SdRegister,
    SdCard,
    SdCardError,
};

pub struct CardId {
    pub manufacturer_id: u8,
    pub oem_id: (char, char),
    pub product_name: (char, char, char, char, char),
    pub product_revision: (u8, u8),
    pub product_serial_num: u32,
    pub manufacturing_date_year: u16,
    pub manufacturing_date_month: u8,
}

impl<CSPIN: avr_hal_generic::port::PinOps> SdCard<CSPIN> {
    pub fn read_card_id(&mut self) -> Result<CardId, SdCardError> {
        match self.read_register(SdRegister::CID) {
            Ok(data) => Ok(CardId {
                manufacturer_id: data[0],
                oem_id: (data[1] as char, data[2] as char),
                product_name: (
                    data[3] as char,
                    data[4] as char,
                    data[5] as char,
                    data[6] as char,
                    data[7] as char,
                ),
                product_revision: (data[8] & 0xf0, data[8] & 0x0f),
                product_serial_num: (data[9] as u32) << 24
                    | (data[10] as u32) << 16
                    | (data[11] as u32) << 8
                    | (data[12] as u32),
                manufacturing_date_year: 2000 + ((data[13] << 4) as u16 | (data[14] >> 4) as u16),
                manufacturing_date_month: data[14] & 0x0f,
            }),
            Err(e) => Err(e),
        }
    }
}
