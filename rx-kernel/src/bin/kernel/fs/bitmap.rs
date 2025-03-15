use bit_field::BitField;

use crate::arch::riscv::qemu::fs::BPB;

use super::{bio::BCache, log::Log, superblock::SuperBlock};

pub struct BitMap {}

impl BitMap {
    /// 找到块号对应的位图的位置
    /// 将该位置置0表示可以重用该块
    pub fn free(dev: u32, blockno: u32) {
        let bm_blockno = SuperBlock::read_bmp();
        let bm_offset = blockno % BPB;
        let index = (bm_offset / 8) as isize;
        let bit = (bm_offset % 8) as usize;
        let mut buf = BCache::read(dev, bm_blockno);
        let byte = unsafe {
            (buf.raw_data_mut() as *mut u8)
                .offset(index)
                .as_mut()
                .unwrap()
        };
        if !byte.get_bit(bit) {
            panic!("bitmap: double freeing a block.");
        }
        byte.set_bit(bit, false);
        Log::write(buf);
    }

    /// 从位图中找一个空闲位
    /// 并返回对应的块号
    pub fn alloc(dev: u32) -> u32 {
        for b in (0..SuperBlock::size()).step_by(BPB as usize) {
            let bm_blockno = SuperBlock::locate_bmp(b);
            let mut buf = BCache::read(dev, bm_blockno);
            for i in 0..BPB.min(SuperBlock::size() - b) {
                let bit = 1 << (i % 8);
                let byte = unsafe {
                    (buf.raw_data_mut() as *mut u8)
                        .offset(i as isize / 8)
                        .as_mut()
                        .unwrap()
                };
                if (*byte & bit) == 0 {
                    *byte |= bit;
                    Log::write(buf);
                    // buf is release now
                    // bzero(dev, b +i);
                    return b + i;
                }
            }
        }
        panic!("bitmap: out of the block ranges.");
    }
}
