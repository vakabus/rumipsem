#[derive(Eq, PartialEq)]
pub enum CPUEvent {
    Nothing,
    Exit,
    AtomicLoadModifyWriteBegan,
}