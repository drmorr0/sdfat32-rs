pub struct File {
    attributes: u8,
    pub(crate) current_cluster: u32,
    pub(crate) current_pos: u32,
    pub(crate) first_cluster: u32,
    flags: u8,
    pub size: u32,
}

const FAT_ATTR_DIRECTORY: u8 = 0x10;

const FILE_ATTR_CLOSED: u8 = 0;
const FILE_ATTR_FILE: u8 = 0x08;
const FILE_ATTR_ROOT: u8 = 0x40;
const FILE_ATTR_SUBDIR: u8 = FAT_ATTR_DIRECTORY;
const FILE_ATTR_DIRECTORY: u8 = FILE_ATTR_SUBDIR | FILE_ATTR_ROOT;

const FILE_FLAG_CONTIGUOUS: u8 = 0x40;

impl File {
    pub fn root_directory() -> File {
        File {
            attributes: FILE_ATTR_CLOSED,
            current_cluster: 0,
            current_pos: 0,
            first_cluster: 0,
            flags: 0,
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
    pub fn is_root(&self) -> bool {
        self.attributes & FILE_ATTR_ROOT > 0
    }
}
