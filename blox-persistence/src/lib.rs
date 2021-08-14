mod db_connection;
mod query_builder;

use db_connection::DbConnection;
use query_builder::QueryBuilder;

pub struct BloxPersistence {
    connection: DbConnection,
}

impl BloxPersistence {
    pub fn query<'a>(&'a self) -> QueryBuilder<'a> {
        QueryBuilder::new(&self.connection)
    }
}
