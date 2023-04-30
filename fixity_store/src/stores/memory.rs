use crate::{
    content_store::{ContentStore, ContentStoreError},
    contentid::Cid,
    mut_store::{MutStore, MutStoreError},
};
use async_trait::async_trait;
use std::{
    collections::{BTreeMap, HashMap},
    ops::RangeBounds,
    sync::{Arc, Mutex},
};

/// A (currently) test focused in-memory only storage.
#[derive(Debug)]
pub struct Memory<Cid> {
    // TODO: change to faster concurrency primitives. At
    // the very least, RwLock instead of Mutex.
    bytes: Mutex<HashMap<Cid, Arc<[u8]>>>,
    mut_: Mutex<BTreeMap<String, Arc<[u8]>>>,
}
#[cfg(any(test, feature = "test"))]
impl Memory<Cid> {
    pub fn test() -> Self {
        Self::default()
    }
}
#[async_trait]
impl ContentStore for Memory<Cid> {
    type Bytes = Arc<[u8]>;
    async fn exists(&self, cid: &Cid) -> Result<bool, ContentStoreError> {
        Ok(self.bytes.lock().unwrap().contains_key(cid))
    }
    async fn read_unchecked(&self, cid: &Cid) -> Result<Self::Bytes, ContentStoreError> {
        let lock = self.bytes.lock().unwrap();
        let buf = lock.get(cid).unwrap();
        Ok(Arc::clone(&buf))
    }
    async fn write_unchecked<B>(&self, cid: &Cid, bytes: B) -> Result<(), ContentStoreError>
    where
        B: AsRef<[u8]> + Into<Arc<[u8]>> + Send,
    {
        let mut lock = self.bytes.lock().unwrap();
        let _ = lock.insert(cid.clone(), bytes.into());
        Ok(())
    }
}
#[async_trait]
impl<Cid> MutStore for Memory<Cid>
where
    Cid: Send,
{
    type Value = Arc<[u8]>;
    async fn list<K, D>(
        &self,
        prefix: K,
        delimiter: Option<D>,
    ) -> Result<Vec<String>, MutStoreError>
    where
        K: AsRef<str> + Send,
        D: AsRef<str> + Send,
    {
        let prefix = prefix.as_ref();
        let mut_ = self.mut_.lock().unwrap();
        if let Some(delimiter) = delimiter {
            let delimiter = delimiter.as_ref();
            // This will fallback to a non-delim when the delim is empty.
            if let Some(last_delim_char) = delimiter.chars().next_back() {
                // NIT: Can this be made to always behave as expected? Also to not
                // fail on saturation, rollover, etc. This area is just a bit weird.
                let after_delimiter =
                    char::from_u32(last_delim_char as u32 + 1).ok_or_else(|| {
                        MutStoreError::InvalidInput {
                            message: String::from("failed to page delim char"),
                        }
                    })?;
                let mut matches = Vec::new();
                delim_cursor(
                    &mut_,
                    prefix,
                    delimiter,
                    after_delimiter,
                    // NIT: This `to_string` is quite painful, however the `range()` API
                    // seems quite awkward for this usecase.
                    prefix.to_string()..,
                    &mut matches,
                );
                return Ok(matches);
            }
        }
        let matches = mut_
            // NIT: This `to_string` is quite painful, however the `range()` API
            // seems quite awkward for this usecase.
            .range(prefix.to_string()..)
            .take_while(|(key, _)| key.starts_with(prefix))
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        Ok(matches)
    }
    async fn get<K>(&self, key: K) -> Result<Self::Value, MutStoreError>
    where
        K: AsRef<str> + Send,
    {
        let lock = self.mut_.lock().unwrap();
        let buf = lock.get(key.as_ref()).ok_or(MutStoreError::NotFound)?;
        Ok(Arc::clone(&buf))
    }
    async fn put<K, V>(&self, key: K, value: V) -> Result<(), MutStoreError>
    where
        K: AsRef<str> + Into<String> + Send,
        V: AsRef<[u8]> + Into<Vec<u8>> + Send,
    {
        let mut mut_ = self.mut_.lock().unwrap();
        let _ = mut_.insert(key.into(), Arc::from(value.into()));
        Ok(())
    }
}
impl<C> Default for Memory<C> {
    fn default() -> Self {
        Self {
            bytes: Default::default(),
            mut_: Default::default(),
        }
    }
}
/// A helper fn to page through a BTreeMap without hitting all the results delimited
/// "folders".
fn delim_cursor<V, R>(
    items: &BTreeMap<String, V>,
    prefix: &str,
    delimiter: &str,
    after_delimiter: char,
    range: R,
    results: &mut Vec<String>,
) where
    R: RangeBounds<String>,
{
    if results.len() == 5 {
        panic!("woo");
    }
    let iter = items
        .range(range)
        .map(|(key, _)| key)
        .take_while(|key| key.starts_with(prefix));
    for item in iter {
        let prefix_stripped = match item.strip_prefix(prefix) {
            Some(s) => s,
            // no resulting strip value means this item equals the prefix,
            // add it because it's a real value and go to the next item.
            None => {
                results.push(prefix.to_string());
                continue;
            },
        };
        // if we get back a split, there's at least one delimiter segment,
        // so add that
        if let Some((delimited_segment, _)) = prefix_stripped.split_once(delimiter) {
            let result = format!("{prefix}{delimited_segment}{delimiter}");
            results.push(result);
            let next_page = format!("{prefix}{delimited_segment}{after_delimiter}");
            return delim_cursor(
                items,
                prefix,
                delimiter,
                after_delimiter,
                next_page..,
                results,
            );
        }
        // if it didn't split, it's not a delimited segment, ie it's just a "file".
        // so push it to results and go to the next item.
        results.push(item.clone());
    }
}
#[cfg(test)]
pub mod test {
    use super::*;
    use rstest::*;
    #[fixture]
    async fn test_data() -> Memory<()> {
        let s = Memory::<()>::default();
        for k in vec![
            "/foo",
            "/foo/bar",
            "/foo/bar/baz",
            "/foo/bar2",
            "/foo/bar3",
            "/foo/bar/baz2",
            "/foo/bar/baz3",
            "/foo/baz",
            "/bong",
        ] {
            s.put(k, "/").await.unwrap();
        }
        s
    }
    #[rstest]
    #[case::delim_none(None)]
    #[case::delim_empty(Some(""))]
    #[tokio::test]
    async fn listing_no_delim(#[case] not_a_delim: Option<&str>) {
        let s: Memory<()> = test_data().await;
        assert_eq!(
            s.list::<_, &str>("/", not_a_delim).await.unwrap(),
            vec![
                "/bong",
                "/foo",
                "/foo/bar",
                "/foo/bar/baz",
                "/foo/bar/baz2",
                "/foo/bar/baz3",
                "/foo/bar2",
                "/foo/bar3",
                "/foo/baz",
            ]
        );
        assert_eq!(
            s.list::<_, &str>("/foo/bar/baz", not_a_delim)
                .await
                .unwrap(),
            vec!["/foo/bar/baz", "/foo/bar/baz2", "/foo/bar/baz3",]
        );
        assert_eq!(
            s.list::<_, &str>("/foo/bar/ba", not_a_delim).await.unwrap(),
            vec!["/foo/bar/baz", "/foo/bar/baz2", "/foo/bar/baz3",]
        );
    }
    #[rstest]
    #[tokio::test]
    async fn listing_delim() {
        let s: Memory<()> = test_data().await;
        let d = Some("/");
        assert_eq!(s.list("/fo", d).await.unwrap(), vec!["/foo", "/foo/"],);
        assert_eq!(s.list("/foo", d).await.unwrap(), vec!["/foo", "/foo/"],);
        assert_eq!(
            s.list("/", d).await.unwrap(),
            vec!["/bong", "/foo", "/foo/"],
        );
        assert_eq!(
            s.list("/foo/", d).await.unwrap(),
            vec![
                "/foo/bar",
                "/foo/bar/",
                "/foo/bar2",
                "/foo/bar3",
                "/foo/baz",
            ],
        );
    }
}
