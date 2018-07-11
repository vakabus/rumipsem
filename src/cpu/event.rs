#[derive(Eq, PartialEq)]
pub enum CPUEvent {
    Nothing,
    Exit,
    AtomicLoadModifyWriteBegan,
    FlowChangeImmediate(u32),
    FlowChangeDelayed(u32),
}