use crate::db::{EMAIL_STATUSES, add_created_at_and_updated_at_timestamps};
use sea_orm_migration::{
    prelude::*,
    schema::{
        integer, pk_uuid, string, string_null, text_null, timestamp_with_time_zone,
        timestamp_with_time_zone_null, uuid, uuid_null,
    },
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(PasswordResetToken::Table)
                    .col(pk_uuid(PasswordResetToken::Id))
                    .col(uuid(PasswordResetToken::UserId))
                    .col(string(PasswordResetToken::TokenHash))
                    .col(timestamp_with_time_zone(PasswordResetToken::ExpiresAt))
                    .col(timestamp_with_time_zone_null(PasswordResetToken::UsedAt))
                    .to_owned(),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_password_reset_token_hash_unique")
                    .table(PasswordResetToken::Table)
                    .col(PasswordResetToken::TokenHash)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_password_reset_token_user")
                    .table(PasswordResetToken::Table)
                    .col(PasswordResetToken::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(EmailOutbox::Table)
                    .col(pk_uuid(EmailOutbox::Id))
                    .col(string(EmailOutbox::Kind))
                    .col(string(EmailOutbox::Recipient))
                    .col(string(EmailOutbox::Subject))
                    .col(text_null(EmailOutbox::EncryptedTextBody))
                    .col(text_null(EmailOutbox::EncryptedHtmlBody))
                    .col(string(EmailOutbox::Status))
                    .col(integer(EmailOutbox::AttemptCount).default(0))
                    .col(timestamp_with_time_zone(EmailOutbox::NextAttemptAt))
                    .col(timestamp_with_time_zone_null(EmailOutbox::LeaseExpiresAt))
                    .col(timestamp_with_time_zone(EmailOutbox::SendBefore))
                    .col(string_null(EmailOutbox::LastError))
                    .col(timestamp_with_time_zone_null(EmailOutbox::SentAt))
                    .col(uuid_null(EmailOutbox::TenantId))
                    .col(uuid_null(EmailOutbox::RequestedByUserId))
                    .check(Expr::col(EmailOutbox::Status).is_in(EMAIL_STATUSES))
                    .to_owned(),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_email_outbox_status_next_attempt")
                    .table(EmailOutbox::Table)
                    .col(EmailOutbox::Status)
                    .col(EmailOutbox::NextAttemptAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_email_outbox_recipient")
                    .table(EmailOutbox::Table)
                    .col(EmailOutbox::Recipient)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_email_outbox_created_at")
                    .table(EmailOutbox::Table)
                    .col(EmailOutbox::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(EmailOutbox::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PasswordResetToken::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PasswordResetToken {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    UsedAt,
}

#[derive(DeriveIden)]
pub enum EmailOutbox {
    Table,
    Id,
    Kind,
    Recipient,
    Subject,
    EncryptedTextBody,
    EncryptedHtmlBody,
    Status,
    AttemptCount,
    NextAttemptAt,
    LeaseExpiresAt,
    SendBefore,
    LastError,
    SentAt,
    TenantId,
    RequestedByUserId,
    CreatedAt,
}
