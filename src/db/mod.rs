pub mod schema;
// pub mod migrations; // TODO: implement migrations

pub use schema::Database;

pub const SCHEMA_VERSION: usize = 1;