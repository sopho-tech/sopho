pub use sea_orm_migration::prelude::*;

mod db;
mod m20250221_000001_create_configuration_table;
mod m20250315_000001_create_dashboard_table;
mod m20250317_000001_create_conversation_table;
mod m20250317_000002_create_conversation_message_content_table;
mod m20250317_000002_create_conversation_message_table;
mod m20250320_000001_create_connection_table;
mod m20250320_000002_create_chart_table;
mod m20250320_000005_create_user_table;
mod m20250404_000001_create_session_table;
mod m20250504_000001_create_notebook_table;
mod m20250510_000001_create_cell_table;
mod m20251210_000001_create_canvas_table;
mod m20260516_000001_create_ai_configuration_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250221_000001_create_configuration_table::Migration),
            Box::new(m20250315_000001_create_dashboard_table::Migration),
            Box::new(m20250320_000001_create_connection_table::Migration),
            Box::new(m20250320_000002_create_chart_table::Migration),
            Box::new(m20250320_000005_create_user_table::Migration),
            Box::new(m20250404_000001_create_session_table::Migration),
            Box::new(m20250504_000001_create_notebook_table::Migration),
            Box::new(m20250510_000001_create_cell_table::Migration),
            Box::new(m20251210_000001_create_canvas_table::Migration),
            Box::new(m20250317_000001_create_conversation_table::Migration),
            Box::new(m20250317_000002_create_conversation_message_table::Migration),
            Box::new(m20250317_000002_create_conversation_message_content_table::Migration),
            Box::new(m20260516_000001_create_ai_configuration_table::Migration),
        ]
    }
}
