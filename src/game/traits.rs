use crate::DenseBools;
use crate::game::BlockFace;

impl From<u8> for DenseBools {
    fn from(item: u8) -> Self {
        Self(item)
    }
}
impl From<[bool; 8]> for DenseBools {
    fn from(item: [bool; 8]) -> Self {
        let mut res: u8 = 0;
        for (i, b) in item.iter().enumerate() { res += if *b { 1 << i } else { 0 } }
        Self( res )
    }
}
impl From<[bool; 7]> for DenseBools {
    fn from(item: [bool; 7]) -> Self {
        let mut res: u8 = 0;
        for (i, b) in item.iter().enumerate() { res += if *b { 1 << i } else { 0 } }
        Self( res )
    }
}
impl From<[bool; 6]> for DenseBools {
    fn from(item: [bool; 6]) -> Self {
        let mut res: u8 = 0;
        for (i, b) in item.iter().enumerate() { res += if *b { 1 << i } else { 0 } }
        Self( res )
    }
}

impl From<BlockFace> for usize {
    fn from(item: BlockFace) -> usize {
        match item {
            BlockFace::PX => 0,
            BlockFace::NX => 1,
            BlockFace::PY => 2,
            BlockFace::NY => 3,
            BlockFace::PZ => 4,
            BlockFace::NZ => 5,
        }
    }
}
impl From<usize> for BlockFace {
    fn from(item: usize) -> Self {
        match item {
            0 => BlockFace::PX,
            1 => BlockFace::NX,
            2 => BlockFace::PY,
            3 => BlockFace::NY,
            4 => BlockFace::PZ,
            5 => BlockFace::NZ,
            i => { panic!("Number {} can't be converted to BlockFace", i) }
        }
    }
}

impl From<BlockFace> for u8 {
    fn from(item: BlockFace) -> u8 {
        match item {
            BlockFace::PX => 0,
            BlockFace::NX => 1,
            BlockFace::PY => 2,
            BlockFace::NY => 3,
            BlockFace::PZ => 4,
            BlockFace::NZ => 5,
        }
    }
}