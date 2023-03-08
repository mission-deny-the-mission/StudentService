pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_course;
mod m20220101_000001_create_student;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220101_000001_create_course::Migration),
            Box::new(m20220101_000001_create_student::Migration)]
    }
}
