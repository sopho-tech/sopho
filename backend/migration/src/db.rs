use sea_orm_migration::{
    prelude::*,
    schema::{string, timestamp_with_time_zone, uuid_null},
};

#[derive(Iden)]
pub enum GeneralIds {
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden, Clone)]
pub enum Attribution {
    CreatedByUserId,
    CreatedByActor,
    ModifiedByUserId,
    ModifiedByActor,
}

pub const ACTOR_KINDS: [&str; 3] = ["HUMAN", "AI", "SYSTEM"];
pub const SYSTEM_ACTOR: &str = "SYSTEM";

pub fn add_created_at_and_updated_at_timestamps(t: TableCreateStatement) -> TableCreateStatement {
    let mut t = t;
    t.col(timestamp_with_time_zone(GeneralIds::CreatedAt).default(Expr::current_timestamp()))
        .col(timestamp_with_time_zone(GeneralIds::UpdatedAt).default(Expr::current_timestamp()))
        .take()
}

pub fn add_attribution_columns(table: TableCreateStatement) -> TableCreateStatement {
    let mut table = table;
    table
        .col(uuid_null(Attribution::CreatedByUserId))
        .col(string(Attribution::CreatedByActor))
        .col(uuid_null(Attribution::ModifiedByUserId))
        .col(string(Attribution::ModifiedByActor))
        .check(Expr::col(Attribution::CreatedByActor).is_in(ACTOR_KINDS))
        .check(person_matches_actor(
            Attribution::CreatedByActor,
            Attribution::CreatedByUserId,
        ))
        .check(Expr::col(Attribution::ModifiedByActor).is_in(ACTOR_KINDS))
        .check(person_matches_actor(
            Attribution::ModifiedByActor,
            Attribution::ModifiedByUserId,
        ))
        .take()
}

fn person_matches_actor(actor_column: Attribution, user_column: Attribution) -> SimpleExpr {
    Expr::col(actor_column.clone())
        .eq(SYSTEM_ACTOR)
        .and(Expr::col(user_column.clone()).is_null())
        .or(Expr::col(actor_column)
            .ne(SYSTEM_ACTOR)
            .and(Expr::col(user_column).is_not_null()))
}
