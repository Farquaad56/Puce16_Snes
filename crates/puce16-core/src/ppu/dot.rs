use crate::consts::{CLOCKS_LONG_LINE, CLOCKS_PER_LINE, CLOCKS_SHORT_LINE};
use crate::region::Region;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LineType { Normal, Short, Long }

pub fn line_type(region: Region, interlace: bool, field: bool, v: u16) -> LineType {
    if region == Region::Ntsc && !interlace && field && v == 240 {
        return LineType::Short;
    }
    if region == Region::Pal && interlace && field && v == 311 {
        return LineType::Long;
    }
    LineType::Normal
}

pub fn line_dots(lt: LineType) -> u16 {
    match lt {
        LineType::Long => 341,
        _ => 340,
    }
}

pub fn line_clocks(lt: LineType) -> u32 {
    match lt {
        LineType::Short => CLOCKS_SHORT_LINE,
        LineType::Long => CLOCKS_LONG_LINE,
        LineType::Normal => CLOCKS_PER_LINE,
    }
}

fn is_five_clock_dot(dot: u16) -> bool {
    dot == 323 || dot == 324 || dot == 327 || dot == 328
}

pub fn dot_clocks(lt: LineType, dot: u16) -> u32 {
    if dot >= line_dots(lt) {
        panic!("dot {} out of range for {:?} (max {})", dot, lt, line_dots(lt));
    }
    if is_five_clock_dot(dot) && lt != LineType::Short {
        5
    } else {
        4
    }
}

pub fn dot_start_clock(lt: LineType, dot: u16) -> u32 {
    let mut sum = 0u32;
    for d in 0..dot {
        sum += dot_clocks(lt, d);
    }
    sum
}
