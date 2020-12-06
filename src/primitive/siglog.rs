
use {super::primitive::AppendLog, crate::Addr, chrono::DateTime};

pub type SigLog<'s, S> = AppendLog<'s, S, Sig>;

#[derive(Debug)]
pub struct Sig {
    pub id: String,
pub sig: String,
    pub addr: Addr,
}
