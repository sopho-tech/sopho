use crate::db::add_created_at_and_updated_at_timestamps;
use sea_orm_migration::{
    prelude::*,
    schema::{boolean, pk_uuid, string, timestamp_with_time_zone, uuid, uuid_null},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(Identity::Table)
                    .col(pk_uuid(Identity::Id))
                    .col(uuid(Identity::UserId))
                    .col(string(Identity::ProviderName))
                    .col(string(Identity::Subject))
                    .col(string(Identity::Email))
                    .col(boolean(Identity::EmailVerified))
                    .to_owned(),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_identity_provider_subject_unique")
                    .table(Identity::Table)
                    .col(Identity::ProviderName)
                    .col(Identity::Subject)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_identity_user")
                    .table(Identity::Table)
                    .col(Identity::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(OidcLoginAttempt::Table)
                    .col(pk_uuid(OidcLoginAttempt::Id))
                    .col(string(OidcLoginAttempt::ProviderName))
                    .col(string(OidcLoginAttempt::State))
                    .col(string(OidcLoginAttempt::Nonce))
                    .col(string(OidcLoginAttempt::CodeVerifier))
                    .col(timestamp_with_time_zone(OidcLoginAttempt::ExpiresAt))
                    .to_owned(),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_oidc_login_attempt_state_unique")
                    .table(OidcLoginAttempt::Table)
                    .col(OidcLoginAttempt::State)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(TenantInvitation::Table)
                    .col(pk_uuid(TenantInvitation::Id))
                    .col(uuid(TenantInvitation::TenantId))
                    .col(string(TenantInvitation::Email))
                    .col(uuid(TenantInvitation::RoleId))
                    .col(string(TenantInvitation::Status))
                    .col(timestamp_with_time_zone(TenantInvitation::ExpiresAt))
                    .col(uuid(TenantInvitation::InvitedByUserId))
                    .col(uuid_null(TenantInvitation::AcceptedByUserId))
                    .to_owned(),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tenant_invitation_tenant_email")
                    .table(TenantInvitation::Table)
                    .col(TenantInvitation::TenantId)
                    .col(TenantInvitation::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tenant_invitation_email")
                    .table(TenantInvitation::Table)
                    .col(TenantInvitation::Email)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TenantInvitation::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(OidcLoginAttempt::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Identity::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Identity {
    Table,
    Id,
    UserId,
    ProviderName,
    Subject,
    Email,
    EmailVerified,
}

#[derive(DeriveIden)]
pub enum OidcLoginAttempt {
    Table,
    Id,
    ProviderName,
    State,
    Nonce,
    CodeVerifier,
    ExpiresAt,
}

#[derive(DeriveIden)]
pub enum TenantInvitation {
    Table,
    Id,
    TenantId,
    Email,
    RoleId,
    Status,
    ExpiresAt,
    InvitedByUserId,
    AcceptedByUserId,
}
