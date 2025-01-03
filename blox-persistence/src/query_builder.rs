use crate::db_connection::DbConnection;

pub struct QueryBuilder<'a> {
    #[allow(dead_code)]
    connection: &'a DbConnection,
}

impl<'a> QueryBuilder<'a> {
    pub fn new(connection: &'a DbConnection) -> Self {
        QueryBuilder { connection }
    }
}
