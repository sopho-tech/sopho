use crate::email_delivery::constants::{EXPIRED_EMAIL_MESSAGE, EmailStatus};
use crate::entity::email_outbox;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sea_query::{Expr, Func};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

pub async fn insert_email(
    connection: &impl ConnectionTrait,
    email: email_outbox::Model,
) -> Result<email_outbox::Model, DbErr> {
    let email_active_model: email_outbox::ActiveModel = email.into();
    email_active_model.insert(connection).await
}

pub async fn find_due_email_ids(
    database_connection: &DatabaseConnection,
    now: DateTimeWithTimeZone,
    limit: u64,
) -> Result<Vec<Uuid>, DbErr> {
    email_outbox::Entity::find()
        .select_only()
        .column(email_outbox::Column::Id)
        .filter(due_for_sending(now))
        .order_by_asc(email_outbox::Column::NextAttemptAt)
        .limit(limit)
        .into_tuple::<Uuid>()
        .all(database_connection)
        .await
}

pub async fn claim_email(
    database_connection: &DatabaseConnection,
    email_id: Uuid,
    now: DateTimeWithTimeZone,
    lease_expires_at: DateTimeWithTimeZone,
) -> Result<Option<email_outbox::Model>, DbErr> {
    let claimed_rows = email_outbox::Entity::update_many()
        .col_expr(
            email_outbox::Column::Status,
            Expr::value(EmailStatus::Sending.as_str()),
        )
        .col_expr(
            email_outbox::Column::LeaseExpiresAt,
            Expr::value(lease_expires_at),
        )
        .col_expr(
            email_outbox::Column::AttemptCount,
            Expr::col(email_outbox::Column::AttemptCount).add(1),
        )
        .col_expr(email_outbox::Column::UpdatedAt, Expr::value(now))
        .filter(email_outbox::Column::Id.eq(email_id))
        .filter(due_for_sending(now))
        .exec(database_connection)
        .await?
        .rows_affected;
    if claimed_rows != 1 {
        return Ok(None);
    }
    email_outbox::Entity::find_by_id(email_id)
        .one(database_connection)
        .await
}

pub async fn mark_email_sent(
    database_connection: &DatabaseConnection,
    email_id: Uuid,
    now: DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(email_outbox::Entity::update_many()
        .col_expr(
            email_outbox::Column::Status,
            Expr::value(EmailStatus::Sent.as_str()),
        )
        .col_expr(email_outbox::Column::SentAt, Expr::value(now))
        .col_expr(email_outbox::Column::LastError, no_text())
        .col_expr(email_outbox::Column::LeaseExpiresAt, no_time())
        .col_expr(email_outbox::Column::EncryptedTextBody, no_text())
        .col_expr(email_outbox::Column::EncryptedHtmlBody, no_text())
        .col_expr(email_outbox::Column::UpdatedAt, Expr::value(now))
        .filter(email_outbox::Column::Id.eq(email_id))
        .filter(email_outbox::Column::Status.eq(EmailStatus::Sending.as_str()))
        .exec(database_connection)
        .await?
        .rows_affected)
}

pub async fn schedule_email_retry(
    database_connection: &DatabaseConnection,
    email_id: Uuid,
    next_attempt_at: DateTimeWithTimeZone,
    last_error: &str,
    now: DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(email_outbox::Entity::update_many()
        .col_expr(
            email_outbox::Column::Status,
            Expr::value(EmailStatus::Pending.as_str()),
        )
        .col_expr(
            email_outbox::Column::NextAttemptAt,
            Expr::value(next_attempt_at),
        )
        .col_expr(email_outbox::Column::LastError, Expr::value(last_error))
        .col_expr(email_outbox::Column::LeaseExpiresAt, no_time())
        .col_expr(email_outbox::Column::UpdatedAt, Expr::value(now))
        .filter(email_outbox::Column::Id.eq(email_id))
        .filter(email_outbox::Column::Status.eq(EmailStatus::Sending.as_str()))
        .exec(database_connection)
        .await?
        .rows_affected)
}

pub async fn mark_email_failed(
    database_connection: &DatabaseConnection,
    email_id: Uuid,
    last_error: &str,
    now: DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(email_outbox::Entity::update_many()
        .col_expr(
            email_outbox::Column::Status,
            Expr::value(EmailStatus::Failed.as_str()),
        )
        .col_expr(email_outbox::Column::LastError, Expr::value(last_error))
        .col_expr(email_outbox::Column::LeaseExpiresAt, no_time())
        .col_expr(email_outbox::Column::UpdatedAt, Expr::value(now))
        .filter(email_outbox::Column::Id.eq(email_id))
        .filter(email_outbox::Column::Status.eq(EmailStatus::Sending.as_str()))
        .exec(database_connection)
        .await?
        .rows_affected)
}

pub async fn mark_email_failed_and_clear_bodies(
    database_connection: &DatabaseConnection,
    email_id: Uuid,
    last_error: &str,
    now: DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(email_outbox::Entity::update_many()
        .col_expr(
            email_outbox::Column::Status,
            Expr::value(EmailStatus::Failed.as_str()),
        )
        .col_expr(email_outbox::Column::LastError, Expr::value(last_error))
        .col_expr(email_outbox::Column::LeaseExpiresAt, no_time())
        .col_expr(email_outbox::Column::EncryptedTextBody, no_text())
        .col_expr(email_outbox::Column::EncryptedHtmlBody, no_text())
        .col_expr(email_outbox::Column::UpdatedAt, Expr::value(now))
        .filter(email_outbox::Column::Id.eq(email_id))
        .filter(email_outbox::Column::Status.eq(EmailStatus::Sending.as_str()))
        .exec(database_connection)
        .await?
        .rows_affected)
}

pub async fn cancel_unsent_email(
    connection: &impl ConnectionTrait,
    email_id: Uuid,
    now: DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(email_outbox::Entity::update_many()
        .col_expr(
            email_outbox::Column::Status,
            Expr::value(EmailStatus::Cancelled.as_str()),
        )
        .col_expr(email_outbox::Column::LeaseExpiresAt, no_time())
        .col_expr(email_outbox::Column::EncryptedTextBody, no_text())
        .col_expr(email_outbox::Column::EncryptedHtmlBody, no_text())
        .col_expr(email_outbox::Column::UpdatedAt, Expr::value(now))
        .filter(email_outbox::Column::Id.eq(email_id))
        .filter(
            email_outbox::Column::Status
                .is_in([EmailStatus::Pending.as_str(), EmailStatus::Failed.as_str()]),
        )
        .exec(connection)
        .await?
        .rows_affected)
}

pub async fn fail_expired_pending_emails(
    database_connection: &DatabaseConnection,
    now: DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(email_outbox::Entity::update_many()
        .col_expr(
            email_outbox::Column::Status,
            Expr::value(EmailStatus::Failed.as_str()),
        )
        .col_expr(
            email_outbox::Column::LastError,
            Expr::value(EXPIRED_EMAIL_MESSAGE),
        )
        .col_expr(email_outbox::Column::LeaseExpiresAt, no_time())
        .col_expr(email_outbox::Column::EncryptedTextBody, no_text())
        .col_expr(email_outbox::Column::EncryptedHtmlBody, no_text())
        .col_expr(email_outbox::Column::UpdatedAt, Expr::value(now))
        .filter(email_outbox::Column::Status.eq(EmailStatus::Pending.as_str()))
        .filter(email_outbox::Column::SendBefore.lte(now))
        .exec(database_connection)
        .await?
        .rows_affected)
}

pub async fn clear_bodies_of_expired_failed_emails(
    database_connection: &DatabaseConnection,
    now: DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(email_outbox::Entity::update_many()
        .col_expr(email_outbox::Column::EncryptedTextBody, no_text())
        .col_expr(email_outbox::Column::EncryptedHtmlBody, no_text())
        .col_expr(email_outbox::Column::UpdatedAt, Expr::value(now))
        .filter(email_outbox::Column::Status.eq(EmailStatus::Failed.as_str()))
        .filter(email_outbox::Column::SendBefore.lte(now))
        .filter(email_outbox::Column::EncryptedTextBody.is_not_null())
        .exec(database_connection)
        .await?
        .rows_affected)
}

pub async fn delete_emails_created_before(
    database_connection: &DatabaseConnection,
    cutoff: DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(email_outbox::Entity::delete_many()
        .filter(email_outbox::Column::CreatedAt.lt(cutoff))
        .exec(database_connection)
        .await?
        .rows_affected)
}

pub async fn count_emails_to_recipient_since(
    connection: &impl ConnectionTrait,
    recipient: &str,
    kinds: Vec<String>,
    since: DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    email_outbox::Entity::find()
        .filter(email_outbox::Column::Recipient.eq(recipient))
        .filter(email_outbox::Column::Kind.is_in(kinds))
        .filter(email_outbox::Column::CreatedAt.gte(since))
        .count(connection)
        .await
}

pub async fn find_emails_by_ids(
    database_connection: &DatabaseConnection,
    email_ids: Vec<Uuid>,
) -> Result<Vec<email_outbox::Model>, DbErr> {
    email_outbox::Entity::find()
        .filter(email_outbox::Column::Id.is_in(email_ids))
        .all(database_connection)
        .await
}

pub struct EmailPage {
    pub emails: Vec<email_outbox::Model>,
    pub total: u64,
}

pub async fn find_emails_page(
    database_connection: &DatabaseConnection,
    status: Option<&str>,
    recipient_search: Option<String>,
    page_index: u64,
    page_size: u64,
) -> Result<EmailPage, DbErr> {
    let emails = email_outbox::Entity::find();
    let emails = match status {
        Some(status) => emails.filter(email_outbox::Column::Status.eq(status)),
        None => emails,
    };
    let emails = match recipient_search {
        Some(recipient_search) => emails.filter(
            Expr::expr(Func::lower(Expr::col(email_outbox::Column::Recipient)))
                .like(format!("%{}%", recipient_search.to_lowercase())),
        ),
        None => emails,
    };
    let paginator = emails
        .order_by_desc(email_outbox::Column::CreatedAt)
        .paginate(database_connection, page_size);
    Ok(EmailPage {
        total: paginator.num_items().await?,
        emails: paginator.fetch_page(page_index).await?,
    })
}

pub async fn find_email(
    database_connection: &DatabaseConnection,
    email_id: Uuid,
) -> Result<Option<email_outbox::Model>, DbErr> {
    email_outbox::Entity::find_by_id(email_id)
        .one(database_connection)
        .await
}

pub async fn requeue_failed_email(
    database_connection: &DatabaseConnection,
    email_id: Uuid,
    now: DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(email_outbox::Entity::update_many()
        .col_expr(
            email_outbox::Column::Status,
            Expr::value(EmailStatus::Pending.as_str()),
        )
        .col_expr(email_outbox::Column::AttemptCount, Expr::value(0))
        .col_expr(email_outbox::Column::NextAttemptAt, Expr::value(now))
        .col_expr(email_outbox::Column::LeaseExpiresAt, no_time())
        .col_expr(email_outbox::Column::UpdatedAt, Expr::value(now))
        .filter(email_outbox::Column::Id.eq(email_id))
        .filter(email_outbox::Column::Status.eq(EmailStatus::Failed.as_str()))
        .filter(email_outbox::Column::EncryptedTextBody.is_not_null())
        .filter(email_outbox::Column::SendBefore.gt(now))
        .exec(database_connection)
        .await?
        .rows_affected)
}

fn due_for_sending(now: DateTimeWithTimeZone) -> Condition {
    Condition::any()
        .add(
            Condition::all()
                .add(email_outbox::Column::Status.eq(EmailStatus::Pending.as_str()))
                .add(email_outbox::Column::NextAttemptAt.lte(now)),
        )
        .add(
            Condition::all()
                .add(email_outbox::Column::Status.eq(EmailStatus::Sending.as_str()))
                .add(email_outbox::Column::LeaseExpiresAt.lte(now)),
        )
}

fn no_text() -> sea_orm::sea_query::SimpleExpr {
    Expr::value(Option::<String>::None)
}

fn no_time() -> sea_orm::sea_query::SimpleExpr {
    Expr::value(Option::<DateTimeWithTimeZone>::None)
}
