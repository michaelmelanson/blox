use blox_interpreter::Value;

mod common;

use common::assert_result;

#[test]
fn test_database() {
    assert_result(
        "
        import { table, column, eq, where, select, to_sql } from 'stdlib/database';
        let users = table(name: 'users');
        users
            .select(column: users.column(name: 'id'))
            .select(column: users.column(name: 'name'))
            .where(condition: users.column(name: 'id').eq(value: '1'))
            .to_sql()
        ",
        Value::String("SELECT users.id, users.name FROM users WHERE users.id = 1".to_string()),
    );
}
