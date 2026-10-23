#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region { Ntsc, Pal }

impl Region {
    pub const fn master_hz(self) -> u64 {
        match self {
            Region::Ntsc => crate::consts::MASTER_HZ_NTSC,
            Region::Pal => crate::consts::MASTER_HZ_PAL,
        }
    }

    pub const fn last_line(self) -> u32 {
        match self {
            Region::Ntsc => 261,
            Region::Pal => 311,
        }
    }

    pub const fn is_pal(self) -> bool {
        match self {
            Region::Ntsc => false,
            Region::Pal => true,
        }
    }
}
