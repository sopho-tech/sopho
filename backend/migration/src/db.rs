use sea_orm_migration::{
    prelude::*,
    schema::{string, timestamp_with_time_zone, uuid_null},
};
use sopho_common::actor_kind::ActorKind;
use sopho_common::email_status::EmailStatus;
use sopho_common::user_status::UserStatus;

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

pub fn actor_kind_names() -> Vec<&'static str> {
    ActorKind::all().iter().map(ActorKind::as_str).collect()
}

pub fn user_status_names() -> Vec<&'static str> {
    UserStatus::all().iter().map(UserStatus::as_str).collect()
}

pub fn email_status_names() -> Vec<&'static str> {
    EmailStatus::all().iter().map(EmailStatus::as_str).collect()
}

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
        .check(Expr::col(Attribution::CreatedByActor).is_in(actor_kind_names()))
        .check(person_matches_actor(
            Attribution::CreatedByActor,
            Attribution::CreatedByUserId,
        ))
        .check(Expr::col(Attribution::ModifiedByActor).is_in(actor_kind_names()))
        .check(person_matches_actor(
            Attribution::ModifiedByActor,
            Attribution::ModifiedByUserId,
        ))
        .take()
}

fn person_matches_actor(actor_column: Attribution, user_column: Attribution) -> SimpleExpr {
    Expr::col(actor_column.clone())
        .eq(ActorKind::System.as_str())
        .and(Expr::col(user_column.clone()).is_null())
        .or(Expr::col(actor_column)
            .ne(ActorKind::System.as_str())
            .and(Expr::col(user_column).is_not_null()))
}
