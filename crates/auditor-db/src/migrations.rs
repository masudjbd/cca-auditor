pub fn get_init_sql() -> &'static str {
    include_str!("../migrations/V1__init.sql")
}
