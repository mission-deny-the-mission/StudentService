use sea_orm_migration::prelude::*;

use crate::m20220101_000001_create_course::Course;
use crate::m20220101_000001_create_student::Student;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Enrolement::Table)
                .if_not_exists()
                .col(ColumnDef::new(Enrolement::StudentId)
                    .integer()
                    .not_null()
                    //.primary_key()
                )
                .col(ColumnDef::new(Enrolement::CourseId)
                    .integer()
                    .not_null()
                    //.primary_key()
                )
                .col(ColumnDef::new(Enrolement::EnrolementDate).string().not_null())
                .foreign_key(ForeignKey::create()
                    .name("fk-student-id")
                    .from(Student::Table, Student::Id)
                    .to(Enrolement::Table, Enrolement::StudentId)
                )
                .foreign_key(ForeignKey::create()
                    .name("fk-course-id")
                    .from(Course::Table, Course::Id)
                    .to(Enrolement::Table, Enrolement::CourseId)
                )
                .primary_key(Index::create()
                    .name("pk-student-course-compound-key")
                    .col(Enrolement::StudentId)
                    .col(Enrolement::CourseId)
                    .primary())
                .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Enrolement::Table).to_owned()).await
    }
}


#[derive(Iden)]
enum Enrolement {
    Table,
    StudentId,
    CourseId,
    EnrolementDate,
}