#[derive(Eq, PartialEq)]
pub enum CPUEvent {
    Nothing,
    Exit,
    AtomicLoadModifyWriteBegan,
    FlowChangeImmediate(u32),   // this is here to support compact branch
    FlowChangeDelayed(u32),
}