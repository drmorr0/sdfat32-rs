use core::default::Default;

#[repr(packed)]
#[derive(Default, Clone, Copy)]
pub struct MbrPartition {
    boot: u8,
    begin_chs: [u8; 3],
    ptype: u8,
    end_chs: [u8; 3],
    relative_sectors: [u8; 4],
    total_sectors: [u8; 4],
}

#[repr(packed)]
pub struct MbrSector {
    boot_code: [u8; 446],
    partitions: [MbrPartition; 4],
    signature: [u8; 2],
}

impl MbrSector {
    pub fn new() -> MbrSector {
        MbrSector {
            boot_code: [0; 446],
            partitions: [Default::default(); 4],
            signature: [0; 2],
        }
    }
}
