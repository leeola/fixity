#[cfg(feature = "borsh")]
use borsh::BorshSerialize;
use {
    crate::{Addr, Commit, Id, Result},
    std::io::{Read, Write},
};
pub trait Store {
    fn put_read(&self, bytes: &mut dyn Read) -> Result<Addr>;
    // fn new() -> Id;
    // fn push(content: T, id: Option<Id>) -> Result<Commit>;
    // fn clone() -> ();
}
#[cfg(feature = "borsh")]
pub trait StoreBorsh {
    fn put<T>(&self, ser: &T) -> Result<Addr>
    where
        T: BorshSerialize;
}
#[cfg(feature = "borsh")]
impl<S> StoreBorsh for S
where
    S: Store,
{
    fn put<T>(&self, ser: &T) -> Result<Addr>
    where
        T: BorshSerialize,
    {
        let mut b = Vec::<u8>::new();
        ser.serialize(&mut b).unwrap();
        self.put_read(&mut &b[..])
    }
}
