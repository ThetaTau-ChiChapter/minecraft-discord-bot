pub use sea_orm_migration::prelude::*;

mod m20260905_180000_initial_migration;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migration_table_name() -> sea_orm::DynIden {
        Alias::new("schema_migrations").into_iden()
    }

    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260905_180000_initial_migration::Migration),
        ]
    }
}
