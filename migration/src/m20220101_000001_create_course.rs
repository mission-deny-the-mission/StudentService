use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
                Table::create()
                .table(Course::Table)
                .if_not_exists()
                .col(ColumnDef::new(Course::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key()
                )
                .col(ColumnDef::new(Course::Name).string().not_null())
                .col(ColumnDef::new(Course::Subject).string().not_null())
                .col(ColumnDef::new(Course::Leader).string().not_null())
                .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Course::Table).to_owned()).await
    }
}

#[derive(Iden)]
enum Course {
    Table,
    Id,
    Name,
    Subject,
    Leader,
}