use blox_interpreter::Value;

mod common;

use common::assert_result;

#[test]
fn test_database() {
    assert_result(
        concat!(
            include_str!("../../stdlib/list.blox"),
            include_str!("../../stdlib/database.blox"),
            "
            let users = table(name: 'users');
            users
              .select(column: users.column(name: 'id'))
              .select(column: users.column(name: 'name'))
              .where(condition: users.column(name: 'id').eq(value: '1'))
              .to_sql()
            "
        ),
        Value::String("SELECT users.id, users.name FROM users WHERE users.id = 1".to_string()),
    );
}
