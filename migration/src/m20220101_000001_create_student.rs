use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Student::Table)
                .if_not_exists()
                .col(ColumnDef::new(Student::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key()
                )
                .col(ColumnDef::new(Student::Name).string().not_null())
                .col(ColumnDef::new(Student::Email).string().not_null())
                .col(ColumnDef::new(Student::StudentID).string().not_null())
                .col(ColumnDef::new(Student::PhoneNumber).string())
                .col(ColumnDef::new(Student::Address).string().not_null())
                .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Student::Table).to_owned()).await
    }
}


#[derive(Iden)]
enum Student {
    Table,
    Id,
    Name,
    Email,
    StudentID,
    PhoneNumber,
    Address,
}