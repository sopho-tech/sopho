use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Configuration::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Configuration::Key)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Configuration::Value).string().not_null())
                    .to_owned(),
            )
            .await?;

        let insert_admin_user_setup = Query::insert()
            .into_table(Configuration::Table)
            .columns([Configuration::Key, Configuration::Value])
            .values_panic(["admin_user_setup".into(), "false".into()])
            .to_owned();

        let insert_demo_data_setup = Query::insert()
            .into_table(Configuration::Table)
            .columns([Configuration::Key, Configuration::Value])
            .values_panic(["demo_data_setup".into(), "false".into()])
            .to_owned();

        manager.exec_stmt(insert_admin_user_setup).await?;
        manager.exec_stmt(insert_demo_data_setup).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Configuration::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Configuration {
    Table,
    Key,
    Value,
}
