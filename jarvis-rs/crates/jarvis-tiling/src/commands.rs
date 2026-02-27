use crate::tree::Direction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TilingCommand {
    SplitHorizontal,
    SplitVertical,
    Close,
    Resize(Direction, i32),
    Swap(Direction),
    FocusNext,
    FocusPrev,
    FocusDirection(Direction),
    Zoom,
}
