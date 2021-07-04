use {
    crate::{
        core::{
            cache::{AsCacheRef, CacheRead, CacheWrite},
            primitive::{BytesCreate, BytesRead},
            workspace::{AsWorkspaceRef, Guard, Workspace},
        },
        Addr, Error, Path,
    },
    gluesql::{GStore, GStoreMut, Row, RowIter, Schema, Store, StoreMut},
    sqlparser::{ast::Statement, dialect::GenericDialect, parser::Parser},
    std::fmt,
    tokio::io::{AsyncRead, AsyncWrite},
};
const KEY_SCHEMA_PREFIX: &str = "schema";
// TODO: Eventually i'd like to try using a custom dialect to allow for time travel
// queries. Hopefully avoiding the need to write my own parser.
const DIALECT: GenericDialect = GenericDialect {};
pub struct Database<'f, C, W> {
    cache: &'f C,
    workspace: &'f W,
    path: Path,
}
impl<'f, C, W> Database<'f, C, W> {
    pub fn new(cache: &'f C, workspace: &'f W, path: Path) -> Self {
        Self {
            cache,
            workspace,
            path,
        }
    }
    pub async fn execute<'s>(&'s self, sql: &str) -> Result<usize, Error>
    where
        C: CacheRead + CacheWrite,
        W: Workspace,
    {
        let glue_store = GlueStore { inner: self };
        let stmt = {
            let mut stmt = gluesql::parse(sql).unwrap();
            // TODO: support multiple ASTs? This is awkward, seems like something Glue should
            // support.
            let stmt = stmt.pop().unwrap();
            gluesql::translate::translate(&stmt).unwrap()
        };
        let (_, payload) = gluesql::execute(glue_store, &stmt).await.unwrap();
        dbg!(payload);
        todo!()
    }
}
impl<'f, C, W> fmt::Debug for Database<'f, C, W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_tuple("Database").field(&self.path).finish()
    }
}
struct GlueStore<'s, 'f, C, W> {
    inner: &'s Database<'f, C, W>,
}
impl<'s, 'f, C, W> GlueStore<'s, 'f, C, W> {}
impl<'s, 'f, C, W> fmt::Debug for GlueStore<'s, 'f, C, W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_tuple("GlueStore").field(&self.inner.path).finish()
    }
}
// wtf is this?
type GlueT = ();
#[async_trait::async_trait(?Send)]
impl<'s, 'f, C, W> Store<GlueT> for GlueStore<'s, 'f, C, W>
where
    C: CacheRead,
    W: Workspace,
{
    async fn fetch_schema(&self, table_name: &str) -> gluesql::Result<Option<Schema>> {
        todo!("fetch_schema")
    }
    async fn scan_data(&self, table_name: &str) -> gluesql::Result<RowIter<GlueT>> {
        todo!("scan_data")
    }
}
#[async_trait::async_trait(?Send)]
impl<'s, 'f, C, W> StoreMut<GlueT> for GlueStore<'s, 'f, C, W>
where
    C: CacheRead + CacheWrite,
    W: Workspace,
{
    async fn insert_schema(self, schema: &Schema) -> gluesql::MutResult<Self, ()> {
        todo!("insert_schema")
    }
    async fn delete_schema(self, table_name: &str) -> gluesql::MutResult<Self, ()> {
        todo!("delete_schema")
    }
    async fn insert_data(self, table_name: &str, rows: Vec<Row>) -> gluesql::MutResult<Self, ()> {
        todo!("insert_data")
    }
    async fn update_data(
        self,
        table_name: &str,
        rows: Vec<(GlueT, Row)>,
    ) -> gluesql::MutResult<Self, ()> {
        todo!("update_data")
    }
    async fn delete_data(self, table_name: &str, keys: Vec<GlueT>) -> gluesql::MutResult<Self, ()> {
        todo!("delete_data")
    }
}
impl<'s, 'f, C, W> GStore<GlueT> for GlueStore<'s, 'f, C, W>
where
    C: CacheRead,
    W: Workspace,
{
}
impl<'s, 'f, C, W> GStoreMut<GlueT> for GlueStore<'s, 'f, C, W>
where
    C: CacheRead + CacheWrite,
    W: Workspace,
{
}
#[cfg(test)]
pub mod test {
    use {super::*, crate::core::Fixity};
    #[tokio::test]
    async fn create_database() {
        let (c, w) = Fixity::memory().into_cw();
        let conn = Database::new(&c, &w, Default::default());
        let row_count = conn
            .execute("CREATE TABLE foo(id INTEGER PRIMARY KEY);")
            .await
            .unwrap();
        dbg!(row_count);
    }
}
