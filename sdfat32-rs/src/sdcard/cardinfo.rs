use super::{
    cmd::SdRegister,
    SdCard,
    SdCardError,
    BLOCK_SIZE,
};

pub struct CardId {
    manufacturer_id: u8,
    oem_id: (u8, u8),
    product_name: [u8; 5],
    product_revision: (u8, u8),
    product_serial_num: u32,
    manufacturing_date_year: u16,
    manufacturing_date_month: u8,
}

impl CardId {
    #[inline(always)]
    pub fn manufacturer_id(&self) -> u8 {
        self.manufacturer_id
    }

    #[inline(always)]
    pub fn oem_id(&self) -> (u8, u8) {
        self.oem_id
    }

    #[inline(always)]
    pub fn product_name(&self) -> [u8; 5] {
        self.product_name
    }

    #[inline(always)]
    pub fn product_revision(&self) -> (u8, u8) {
        self.product_revision
    }

    #[inline(always)]
    pub fn product_serial_num(&self) -> u32 {
        self.product_serial_num
    }

    #[inline(always)]
    pub fn manufacturing_date(&self) -> (u16, u8) {
        (self.manufacturing_date_year, self.manufacturing_date_month)
    }
}

pub struct CardSpecificData {
    version: u8,
    tran_speed_mhz: u8,
    supported_command_classes: u16,
    max_read_block_len_bytes: usize,
    capacity_mib: u32,
}

impl CardSpecificData {
    #[inline(always)]
    pub fn version(&self) -> u8 {
        self.version
    }

    #[inline(always)]
    pub fn tran_speed_mhz(&self) -> u8 {
        self.tran_speed_mhz
    }

    #[inline(always)]
    pub fn supported_command_classes(&self) -> u16 {
        self.supported_command_classes
    }

    #[inline(always)]
    pub fn max_read_block_len_bytes(&self) -> usize {
        self.max_read_block_len_bytes
    }

    #[inline(always)]
    pub fn capacity_mib(&self) -> u32 {
        self.capacity_mib
    }
}

impl<CSPIN: avr_hal_generic::port::PinOps> SdCard<CSPIN> {
    pub fn read_card_id(&mut self) -> Result<CardId, SdCardError> {
        let data = self.read_register(SdRegister::CID)?;
        Ok(CardId {
            manufacturer_id: data[0],
            oem_id: (data[1], data[2]),
            product_name: [data[3], data[4], data[5], data[6], data[7]],
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
                0x09 => BLOCK_SIZE,
                _ => 0,
            },
            capacity_mib: ((((((data[7] & 0x3f) as u32) << 16 | (data[8] as u32) << 8 | (data[9] as u32)) + 1) as u64
                * 512000)
                >> 20) as u32,
        })
    }
}
