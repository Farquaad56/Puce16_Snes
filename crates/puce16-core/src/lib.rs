#![forbid(unsafe_code)]

pub mod consts;
pub mod region;
pub mod snes;
pub mod scheduler;
pub mod cart;
pub mod bus;
pub mod cpu;
pub mod dma;
pub mod timing;
pub mod ppu;
pub mod apu;
pub mod joypad;
pub mod savestate;
pub mod hash;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use region::Region;
pub use snes::Snes;
pub use ppu::output::FrameView;
