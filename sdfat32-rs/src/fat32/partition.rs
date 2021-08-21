
const BYTES_PER_SECTOR_SHIFT: u8 = 9;
const BYTES_PER_SECTOR: u16 = 512;
const SECTOR_MASK: u16 = 0x1FF;

struct Partition {
    sectors_per_cluster: u8,
    cluster_sector_mask: u8,
    sectors_per_cluster_shift: u8,
    allocSearchStart: u32,
    sectorsPerFat: u32,
    dataStartSector: u32,
    fatStartSector: u32,
    lastCluster: u32,
    rootDirStart: u32,
};


