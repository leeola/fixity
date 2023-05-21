
use async_trait::async_trait;
use std::{sync::Arc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MutStoreError {
    #[error("resource not found")]
    NotFound,
    #[error("invalid input: {message}")]
    InvalidInput { message: String },
}

#[async_trait]
pub trait MutStore: Send + Sync {
    type Value: AsRef<[u8]> + Into<Arc<[u8]>>;
    async fn list<K, D>(
        &self,
        prefix: K,
        delimiter: Option<D>,
    ) -> Result<Vec<String>, MutStoreError>
    where
        K: AsRef<str> + Send,
        D: AsRef<str> + Send;
    async fn get<K>(&self, key: K) -> Result<Self::Value, MutStoreError>
    where
        K: AsRef<str> + Send;
    async fn put<K, V>(&self, key: K, value: V) -> Result<(), MutStoreError>
    where
        K: AsRef<str> + Into<String> + Send,
        V: AsRef<[u8]> + Into<Vec<u8>> + Send;
}
