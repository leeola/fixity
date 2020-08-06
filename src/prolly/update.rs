pub enum Update {
    Insert,
    Remove,
    Replace,
}

pub struct Tree {
    updates: Vec<Update>,
}
