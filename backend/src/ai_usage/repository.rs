use crate::ai_usage::constants::{
    AGENT_NAME_ALIAS, BIG_INTEGER_TYPE, CACHED_INPUT_TOKENS_ALIAS, CALL_COUNT_ALIAS,
    CONVERSATION_ID_ALIAS, CONVERSATION_ID_EXPRESSION, CONVERSATION_MESSAGE_ID_ALIAS,
    CONVERSATION_MESSAGE_ID_EXPRESSION, DAY_ALIAS, FIRST_CALL_AT_ALIAS, INPUT_TOKENS_ALIAS,
    OUTPUT_TOKENS_ALIAS, POSTGRES_UTC_DAY_EXPRESSION, SQLITE_UTC_DAY_EXPRESSION, TENANT_ID_ALIAS,
    USER_ID_ALIAS,
};
use crate::ai_usage::datatypes::{UsageRange, UsageScope};
use crate::authorization::dto::TenantId;
use crate::entity::{ai_usage, platform_ai_usage};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sea_query::{Alias, Expr, ExprTrait, Func, OnConflict, SimpleExpr};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbBackend,
    DbErr, EntityTrait, FromQueryResult, Order, QueryFilter, QueryOrder, QuerySelect, Select, Set,
};
use uuid::Uuid;

#[derive(Debug, FromQueryResult)]
pub struct UsageTotalsRow {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cached_input_tokens: i64,
    pub call_count: i64,
}

#[derive(Debug, FromQueryResult)]
pub struct UsageDayRow {
    pub day: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub call_count: i64,
}

#[derive(Debug, FromQueryResult)]
pub struct UsageUserRow {
    pub user_id: Option<Uuid>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub call_count: i64,
}

#[derive(Debug, FromQueryResult)]
pub struct UsageAgentRow {
    pub agent_name: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub call_count: i64,
}

#[derive(Debug, FromQueryResult)]
pub struct UsageConversationRow {
    pub conversation_id: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub call_count: i64,
}

#[derive(Debug, FromQueryResult)]
pub struct UsageConversationUserRow {
    pub conversation_id: String,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

#[derive(Debug, FromQueryResult)]
pub struct UsageQuestionRow {
    pub conversation_message_id: Option<String>,
    pub first_call_at: DateTimeWithTimeZone,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub call_count: i64,
}

#[derive(Debug, FromQueryResult)]
pub struct UsageTenantRow {
    pub tenant_id: Uuid,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub call_count: i64,
}

#[derive(Debug, FromQueryResult)]
pub struct UsageTenantUserRow {
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub call_count: i64,
}

pub async fn insert_ai_usage(
    database_connection: &impl ConnectionTrait,
    ai_usage_row: ai_usage::ActiveModel,
) -> Result<ai_usage::Model, DbErr> {
    ai_usage_row.insert(database_connection).await
}

pub async fn add_platform_ai_tokens(
    database_connection: &impl ConnectionTrait,
    tenant_id: &TenantId,
    used_tokens: i64,
    updated_at: DateTimeWithTimeZone,
) -> Result<(), DbErr> {
    platform_ai_usage::Entity::insert(platform_ai_usage::ActiveModel {
        tenant_id: Set(tenant_id.get_uuid()),
        used_tokens: Set(used_tokens),
        updated_at: Set(updated_at),
    })
    .on_conflict(
        OnConflict::column(platform_ai_usage::Column::TenantId)
            .value(
                platform_ai_usage::Column::UsedTokens,
                Expr::col((
                    platform_ai_usage::Entity,
                    platform_ai_usage::Column::UsedTokens,
                ))
                .add(used_tokens),
            )
            .value(platform_ai_usage::Column::UpdatedAt, updated_at)
            .to_owned(),
    )
    .exec(database_connection)
    .await?;
    Ok(())
}

pub async fn find_platform_ai_usage(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
) -> Result<Option<platform_ai_usage::Model>, DbErr> {
    platform_ai_usage::Entity::find_by_id(tenant_id.get_uuid())
        .one(database_connection)
        .await
}

pub async fn list_platform_ai_usage(
    database_connection: &DatabaseConnection,
) -> Result<Vec<platform_ai_usage::Model>, DbErr> {
    platform_ai_usage::Entity::find()
        .all(database_connection)
        .await
}

fn summed_tokens(token_column: ai_usage::Column) -> SimpleExpr {
    Expr::expr(Func::coalesce([
        Expr::col(token_column).sum(),
        Expr::val(0_i64),
    ]))
    .cast_as(Alias::new(BIG_INTEGER_TYPE))
}

fn total_tokens_ordering() -> SimpleExpr {
    Expr::col(ai_usage::Column::InputTokens)
        .sum()
        .add(Expr::col(ai_usage::Column::OutputTokens).sum())
}

fn conversation_id_expression() -> SimpleExpr {
    Expr::cust(CONVERSATION_ID_EXPRESSION)
}

fn conversation_message_id_expression() -> SimpleExpr {
    Expr::cust(CONVERSATION_MESSAGE_ID_EXPRESSION)
}

fn utc_day_expression(database_backend: DbBackend) -> Result<SimpleExpr, DbErr> {
    match database_backend {
        DbBackend::Sqlite => Ok(Expr::cust(SQLITE_UTC_DAY_EXPRESSION)),
        DbBackend::Postgres => Ok(Expr::cust(POSTGRES_UTC_DAY_EXPRESSION)),
        unsupported_database_backend => Err(DbErr::Custom(format!(
            "AI usage by day is not supported on {unsupported_database_backend:?}"
        ))),
    }
}

fn usage_in_range(
    usage_scope: UsageScope<'_>,
    usage_range: &UsageRange,
) -> Select<ai_usage::Entity> {
    let starts_at: DateTimeWithTimeZone = usage_range.starts_at.into();
    let ends_before: DateTimeWithTimeZone = usage_range.ends_before.into();
    let mut usage_condition = Condition::all()
        .add(ai_usage::Column::CreatedAt.gte(starts_at))
        .add(ai_usage::Column::CreatedAt.lt(ends_before));
    if let UsageScope::Tenant { tenant_id } = usage_scope {
        usage_condition = usage_condition.add(ai_usage::Column::TenantId.eq(tenant_id.get_uuid()));
    }
    if let Some(key_source) = usage_range.key_source {
        let key_source: &'static str = key_source.into();
        usage_condition = usage_condition.add(ai_usage::Column::KeySource.eq(key_source));
    }
    ai_usage::Entity::find()
        .select_only()
        .filter(usage_condition)
}

fn with_token_sums(usage_query: Select<ai_usage::Entity>) -> Select<ai_usage::Entity> {
    usage_query
        .column_as(
            summed_tokens(ai_usage::Column::InputTokens),
            INPUT_TOKENS_ALIAS,
        )
        .column_as(
            summed_tokens(ai_usage::Column::OutputTokens),
            OUTPUT_TOKENS_ALIAS,
        )
}

fn with_call_count(usage_query: Select<ai_usage::Entity>) -> Select<ai_usage::Entity> {
    usage_query.column_as(ai_usage::Column::Id.count(), CALL_COUNT_ALIAS)
}

pub async fn sum_usage(
    database_connection: &DatabaseConnection,
    usage_scope: UsageScope<'_>,
    usage_range: &UsageRange,
) -> Result<Option<UsageTotalsRow>, DbErr> {
    with_call_count(with_token_sums(usage_in_range(usage_scope, usage_range)))
        .column_as(
            summed_tokens(ai_usage::Column::CachedInputTokens),
            CACHED_INPUT_TOKENS_ALIAS,
        )
        .into_model::<UsageTotalsRow>()
        .one(database_connection)
        .await
}

pub async fn count_calls_without_token_counts(
    database_connection: &DatabaseConnection,
    usage_scope: UsageScope<'_>,
    usage_range: &UsageRange,
) -> Result<Option<UsageTotalsRow>, DbErr> {
    with_call_count(with_token_sums(usage_in_range(usage_scope, usage_range)))
        .column_as(
            summed_tokens(ai_usage::Column::CachedInputTokens),
            CACHED_INPUT_TOKENS_ALIAS,
        )
        .filter(ai_usage::Column::InputTokens.eq(0))
        .filter(ai_usage::Column::OutputTokens.eq(0))
        .into_model::<UsageTotalsRow>()
        .one(database_connection)
        .await
}

pub async fn sum_usage_by_day(
    database_connection: &DatabaseConnection,
    usage_scope: UsageScope<'_>,
    usage_range: &UsageRange,
) -> Result<Vec<UsageDayRow>, DbErr> {
    let utc_day = utc_day_expression(database_connection.get_database_backend())?;
    with_call_count(with_token_sums(usage_in_range(usage_scope, usage_range)))
        .column_as(utc_day.clone(), DAY_ALIAS)
        .group_by(utc_day)
        .into_model::<UsageDayRow>()
        .all(database_connection)
        .await
}

pub async fn sum_usage_by_user(
    database_connection: &DatabaseConnection,
    usage_scope: UsageScope<'_>,
    usage_range: &UsageRange,
) -> Result<Vec<UsageUserRow>, DbErr> {
    with_call_count(with_token_sums(usage_in_range(usage_scope, usage_range)))
        .column_as(ai_usage::Column::UserId, USER_ID_ALIAS)
        .group_by(ai_usage::Column::UserId)
        .order_by(total_tokens_ordering(), Order::Desc)
        .into_model::<UsageUserRow>()
        .all(database_connection)
        .await
}

pub async fn sum_usage_by_agent(
    database_connection: &DatabaseConnection,
    usage_scope: UsageScope<'_>,
    usage_range: &UsageRange,
) -> Result<Vec<UsageAgentRow>, DbErr> {
    with_call_count(with_token_sums(usage_in_range(usage_scope, usage_range)))
        .column_as(ai_usage::Column::AgentName, AGENT_NAME_ALIAS)
        .group_by(ai_usage::Column::AgentName)
        .into_model::<UsageAgentRow>()
        .all(database_connection)
        .await
}

pub async fn sum_usage_by_conversation(
    database_connection: &DatabaseConnection,
    usage_scope: UsageScope<'_>,
    usage_range: &UsageRange,
) -> Result<Vec<UsageConversationRow>, DbErr> {
    with_call_count(with_token_sums(usage_in_range(usage_scope, usage_range)))
        .column_as(conversation_id_expression(), CONVERSATION_ID_ALIAS)
        .filter(Expr::expr(conversation_id_expression()).is_not_null())
        .group_by(conversation_id_expression())
        .order_by(total_tokens_ordering(), Order::Desc)
        .order_by(conversation_id_expression(), Order::Asc)
        .into_model::<UsageConversationRow>()
        .all(database_connection)
        .await
}

pub async fn sum_usage_by_conversation_and_user(
    database_connection: &DatabaseConnection,
    usage_scope: UsageScope<'_>,
    usage_range: &UsageRange,
) -> Result<Vec<UsageConversationUserRow>, DbErr> {
    with_token_sums(usage_in_range(usage_scope, usage_range))
        .column_as(conversation_id_expression(), CONVERSATION_ID_ALIAS)
        .column_as(ai_usage::Column::TenantId, TENANT_ID_ALIAS)
        .column_as(ai_usage::Column::UserId, USER_ID_ALIAS)
        .filter(Expr::expr(conversation_id_expression()).is_not_null())
        .group_by(conversation_id_expression())
        .group_by(ai_usage::Column::TenantId)
        .group_by(ai_usage::Column::UserId)
        .into_model::<UsageConversationUserRow>()
        .all(database_connection)
        .await
}

pub async fn sum_usage_by_question(
    database_connection: &DatabaseConnection,
    usage_scope: UsageScope<'_>,
    usage_range: &UsageRange,
    conversation_id: Uuid,
) -> Result<Vec<UsageQuestionRow>, DbErr> {
    with_call_count(with_token_sums(usage_in_range(usage_scope, usage_range)))
        .column_as(
            conversation_message_id_expression(),
            CONVERSATION_MESSAGE_ID_ALIAS,
        )
        .column_as(ai_usage::Column::CreatedAt.min(), FIRST_CALL_AT_ALIAS)
        .filter(Expr::expr(conversation_id_expression()).eq(conversation_id.to_string()))
        .group_by(conversation_message_id_expression())
        .order_by(ai_usage::Column::CreatedAt.min(), Order::Asc)
        .into_model::<UsageQuestionRow>()
        .all(database_connection)
        .await
}

pub async fn sum_usage_by_tenant(
    database_connection: &DatabaseConnection,
    usage_range: &UsageRange,
) -> Result<Vec<UsageTenantRow>, DbErr> {
    with_call_count(with_token_sums(usage_in_range(
        UsageScope::AllTenants,
        usage_range,
    )))
    .column_as(ai_usage::Column::TenantId, TENANT_ID_ALIAS)
    .group_by(ai_usage::Column::TenantId)
    .order_by(total_tokens_ordering(), Order::Desc)
    .order_by(ai_usage::Column::TenantId, Order::Asc)
    .into_model::<UsageTenantRow>()
    .all(database_connection)
    .await
}

pub async fn sum_usage_by_tenant_and_user(
    database_connection: &DatabaseConnection,
    usage_range: &UsageRange,
) -> Result<Vec<UsageTenantUserRow>, DbErr> {
    with_call_count(with_token_sums(usage_in_range(
        UsageScope::AllTenants,
        usage_range,
    )))
    .column_as(ai_usage::Column::TenantId, TENANT_ID_ALIAS)
    .column_as(ai_usage::Column::UserId, USER_ID_ALIAS)
    .group_by(ai_usage::Column::TenantId)
    .group_by(ai_usage::Column::UserId)
    .order_by(total_tokens_ordering(), Order::Desc)
    .order_by(ai_usage::Column::TenantId, Order::Asc)
    .order_by(ai_usage::Column::UserId, Order::Asc)
    .into_model::<UsageTenantUserRow>()
    .all(database_connection)
    .await
}
