use crate::ai_usage::constants::{
    AGENT_NAME_ALIAS, BIG_INTEGER_TYPE, CACHED_INPUT_TOKENS_ALIAS, CALL_COUNT_ALIAS,
    CONVERSATION_COUNT_ALIAS, CONVERSATION_ID_ALIAS, CONVERSATION_ID_EXPRESSION,
    CONVERSATION_MESSAGE_ID_ALIAS, CONVERSATION_MESSAGE_ID_EXPRESSION, DAY_ALIAS,
    FIRST_CALL_AT_ALIAS, INPUT_TOKENS_ALIAS, OUTPUT_TOKENS_ALIAS, POSTGRES_UTC_DAY_EXPRESSION,
    SQLITE_UTC_DAY_EXPRESSION, USER_ID_ALIAS,
};
use crate::ai_usage::dto::UsageRange;
use crate::authorization::dto::TenantId;
use crate::entity::ai_usage;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sea_query::{Alias, Expr, Func, SimpleExpr};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbBackend,
    DbErr, EntityTrait, FromQueryResult, Order, QueryFilter, QueryOrder, QuerySelect, Select,
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
struct ConversationCountRow {
    conversation_count: i64,
}

pub async fn insert_ai_usage(
    database_connection: &DatabaseConnection,
    ai_usage_row: ai_usage::ActiveModel,
) -> Result<ai_usage::Model, DbErr> {
    ai_usage_row.insert(database_connection).await
}

fn summed_tokens(token_column: ai_usage::Column) -> SimpleExpr {
    Expr::expr(Func::coalesce([
        Expr::col(token_column).sum(),
        Expr::val(0_i64).into(),
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
        DbBackend::MySql => Err(DbErr::Custom(
            "AI usage by day is not supported on MySQL".to_string(),
        )),
    }
}

fn usage_in_range(tenant_id: &TenantId, usage_range: &UsageRange) -> Select<ai_usage::Entity> {
    let starts_at: DateTimeWithTimeZone = usage_range.starts_at.into();
    let ends_before: DateTimeWithTimeZone = usage_range.ends_before.into();
    ai_usage::Entity::find().select_only().filter(
        Condition::all()
            .add(ai_usage::Column::TenantId.eq(tenant_id.get_uuid()))
            .add(ai_usage::Column::CreatedAt.gte(starts_at))
            .add(ai_usage::Column::CreatedAt.lt(ends_before)),
    )
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
    tenant_id: &TenantId,
    usage_range: &UsageRange,
) -> Result<Option<UsageTotalsRow>, DbErr> {
    with_call_count(with_token_sums(usage_in_range(tenant_id, usage_range)))
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
    tenant_id: &TenantId,
    usage_range: &UsageRange,
) -> Result<Option<UsageTotalsRow>, DbErr> {
    with_call_count(with_token_sums(usage_in_range(tenant_id, usage_range)))
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
    tenant_id: &TenantId,
    usage_range: &UsageRange,
) -> Result<Vec<UsageDayRow>, DbErr> {
    let utc_day = utc_day_expression(database_connection.get_database_backend())?;
    with_call_count(with_token_sums(usage_in_range(tenant_id, usage_range)))
        .column_as(utc_day.clone(), DAY_ALIAS)
        .group_by(utc_day)
        .into_model::<UsageDayRow>()
        .all(database_connection)
        .await
}

pub async fn sum_usage_by_user(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    usage_range: &UsageRange,
) -> Result<Vec<UsageUserRow>, DbErr> {
    with_call_count(with_token_sums(usage_in_range(tenant_id, usage_range)))
        .column_as(ai_usage::Column::UserId, USER_ID_ALIAS)
        .group_by(ai_usage::Column::UserId)
        .order_by(total_tokens_ordering(), Order::Desc)
        .into_model::<UsageUserRow>()
        .all(database_connection)
        .await
}

pub async fn sum_usage_by_agent(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    usage_range: &UsageRange,
) -> Result<Vec<UsageAgentRow>, DbErr> {
    with_call_count(with_token_sums(usage_in_range(tenant_id, usage_range)))
        .column_as(ai_usage::Column::AgentName, AGENT_NAME_ALIAS)
        .group_by(ai_usage::Column::AgentName)
        .into_model::<UsageAgentRow>()
        .all(database_connection)
        .await
}

pub async fn count_conversations_with_usage(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    usage_range: &UsageRange,
) -> Result<Option<i64>, DbErr> {
    let conversation_count = usage_in_range(tenant_id, usage_range)
        .column_as(
            Expr::expr(conversation_id_expression()).count_distinct(),
            CONVERSATION_COUNT_ALIAS,
        )
        .filter(Expr::expr(conversation_id_expression()).is_not_null())
        .into_model::<ConversationCountRow>()
        .one(database_connection)
        .await?;
    Ok(conversation_count.map(|row| row.conversation_count))
}

pub async fn sum_usage_by_conversation(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    usage_range: &UsageRange,
    first_row_offset: u64,
    page_size: u64,
) -> Result<Vec<UsageConversationRow>, DbErr> {
    with_call_count(with_token_sums(usage_in_range(tenant_id, usage_range)))
        .column_as(conversation_id_expression(), CONVERSATION_ID_ALIAS)
        .filter(Expr::expr(conversation_id_expression()).is_not_null())
        .group_by(conversation_id_expression())
        .order_by(total_tokens_ordering(), Order::Desc)
        .order_by(conversation_id_expression(), Order::Asc)
        .offset(first_row_offset)
        .limit(page_size)
        .into_model::<UsageConversationRow>()
        .all(database_connection)
        .await
}

pub async fn sum_usage_by_conversation_and_user(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    usage_range: &UsageRange,
    conversation_ids: &[String],
) -> Result<Vec<UsageConversationUserRow>, DbErr> {
    with_token_sums(usage_in_range(tenant_id, usage_range))
        .column_as(conversation_id_expression(), CONVERSATION_ID_ALIAS)
        .column_as(ai_usage::Column::UserId, USER_ID_ALIAS)
        .filter(Expr::expr(conversation_id_expression()).is_in(conversation_ids.to_vec()))
        .group_by(conversation_id_expression())
        .group_by(ai_usage::Column::UserId)
        .into_model::<UsageConversationUserRow>()
        .all(database_connection)
        .await
}

pub async fn sum_usage_by_question(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    usage_range: &UsageRange,
    conversation_id: Uuid,
) -> Result<Vec<UsageQuestionRow>, DbErr> {
    with_call_count(with_token_sums(usage_in_range(tenant_id, usage_range)))
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
