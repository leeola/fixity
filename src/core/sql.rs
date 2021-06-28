use {
    crate::{
        core::{
            cache::{AsCacheRef, CacheRead, CacheWrite},
            primitive::{BytesCreate, BytesRead},
            workspace::{AsWorkspaceRef, Guard, Workspace},
        },
        Addr, Error, Path,
    },
    sqlparser::{ast::Statement, dialect::GenericDialect, parser::Parser},
    tokio::io::{AsyncRead, AsyncWrite},
};
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
    pub fn query<'s>(&'s self, sql: &str) -> Result<Query<'s, 'f, C, W>, Error> {
        let ast = Parser::parse_sql(&DIALECT, sql).unwrap();
        dbg!(&ast);
        Ok(Query { inner: self, ast })
    }
}
pub struct Query<'s, 'f, C, W> {
    inner: &'s Database<'f, C, W>,
    ast: Vec<Statement>,
}
impl<'s, 'f, C, W> Query<'s, 'f, C, W> {
    pub async fn execute(self) -> Result<usize, Error> {
        todo!()
    }
}
#[cfg(test)]
pub mod test {
    use {super::*, crate::core::Fixity};
    #[tokio::test]
    async fn create_database() {
        let (c, w) = Fixity::memory().into_cw();
        let conn = Database::new(&c, &w, Default::default());
        let query = conn
            .query("CREATE TABLE foo(id INTEGER PRIMARY KEY);")
            .unwrap();
        query.execute().await.unwrap();
    }
}
