//! Events that a single instruction can trigger. They are internal to the emulator and not in any
//! way related to exceptions in the actual MIPS CPU.

#[derive(Eq, PartialEq)]
pub enum CPUEvent {
    Nothing,
    Exit,
    AtomicLoadModifyWriteBegan,
    Fork(u32),
    FlowChangeImmediate(u32), // this is here to support compact branch
    FlowChangeDelayed(u32),
}
