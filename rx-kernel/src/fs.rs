#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InodeType {
    Empty = 0,
    Directory = 1,
    File = 2,
    Device = 3,
}

#[repr(C)]
pub struct Stat {
    pub dev: u32,         // file
    pub inum: u32,        // Inode number
    pub itype: InodeType, // Type of file
    pub nlink: i16,       // Number of links to link
    pub size: usize,      // Size of file bytes
}

impl Stat {
    pub const fn new() -> Self {
        Self {
            dev: 0,
            inum: 0,
            itype: InodeType::Empty,
            nlink: 0,
            size: 0,
        }
    }
}

impl Default for Stat {
    fn default() -> Self {
        Self::new()
    }
}
