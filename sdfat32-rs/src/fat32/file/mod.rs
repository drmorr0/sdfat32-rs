mod dir;

pub struct File {
    pub(crate) vol_id: u8,
    attributes: u8,
    pub(crate) cluster: u32,
    pub(crate) pos: u32,
    pub(crate) start_cluster: u32,
    flags: u8,
    size: u32,
}

const FAT_ATTR_DIRECTORY: u8 = 0x10;

const FILE_ATTR_CLOSED: u8 = 0;
const FILE_ATTR_FILE: u8 = 0x08;
const FILE_ATTR_ROOT: u8 = 0x40;
const FILE_ATTR_SUBDIR: u8 = FAT_ATTR_DIRECTORY;
const FILE_ATTR_DIRECTORY: u8 = FILE_ATTR_SUBDIR | FILE_ATTR_ROOT;

const FILE_FLAG_READ: u8 = 0x01;
const FILE_FLAG_WRITE: u8 = 0x02;
const FILE_FLAG_CONTIGUOUS: u8 = 0x40;

impl File {
    pub(crate) fn open_root(vol_id: u8) -> File {
        File {
            vol_id,
            attributes: FILE_ATTR_ROOT,
            cluster: 0,
            pos: 0,
            start_cluster: 0,
            flags: FILE_FLAG_READ,
            size: 0,
        }
    }

    #[inline(always)]
    pub fn is_contiguous(&self) -> bool {
        self.flags & FILE_FLAG_CONTIGUOUS > 0
    }

    #[inline(always)]
    pub fn is_directory(&self) -> bool {
        self.attributes & FILE_ATTR_DIRECTORY > 0
    }

    #[inline(always)]
    pub fn is_file(&self) -> bool {
        self.attributes & FILE_ATTR_FILE > 0
    }

    #[inline(always)]
    pub fn is_open(&self) -> bool {
        self.attributes > 0
    }

    #[inline(always)]
    pub fn is_readable(&self) -> bool {
        self.flags & FILE_FLAG_READ > 0
    }

    #[inline(always)]
    pub fn is_root(&self) -> bool {
        self.attributes & FILE_ATTR_ROOT > 0
    }

    #[inline(always)]
    pub fn size(&self) -> u32 {
        self.size
    }
}
