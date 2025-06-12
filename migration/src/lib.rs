pub use sea_orm_migration::prelude::*;

mod m20250611_142906_create_board_data_table;
mod m20250612_035646_create_vote;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250611_142906_create_board_data_table::Migration),
            Box::new(m20250612_035646_create_vote::Migration),
        ]
    }
}
