use sea_orm_migration::{prelude::*, schema::timestamp_with_time_zone};

#[derive(Iden)]
pub enum GeneralIds {
    CreatedAt,
    UpdatedAt,
}

pub fn add_created_at_and_updated_at_timestamps(t: TableCreateStatement) -> TableCreateStatement {
    let mut t = t;
    t.col(timestamp_with_time_zone(GeneralIds::CreatedAt).default(Expr::current_timestamp()))
        .col(timestamp_with_time_zone(GeneralIds::UpdatedAt).default(Expr::current_timestamp()))
        .take()
}
