use {super::primitive::AppendLog, crate::Addr, chrono::DateTime};

pub type CommitLog<'s, S> = AppendLog<'s, S, Commit>;

#[derive(Debug)]
pub struct Commit {
    pub date: DateTime,
    pub addr: Addr,
}
