use crate::db::add_created_at_and_updated_at_timestamps;
use sea_orm_migration::{
    prelude::*,
    schema::{boolean, pk_uuid, string, string_null, timestamp_with_time_zone, uuid, uuid_null},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(FederatedIdentity::Table)
                    .col(pk_uuid(FederatedIdentity::Id))
                    .col(uuid(FederatedIdentity::UserId))
                    .col(string(FederatedIdentity::ProviderName))
                    .col(string(FederatedIdentity::Subject))
                    .col(string(FederatedIdentity::Email))
                    .col(boolean(FederatedIdentity::EmailVerified))
                    .to_owned(),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_federated_identity_provider_subject_unique")
                    .table(FederatedIdentity::Table)
                    .col(FederatedIdentity::ProviderName)
                    .col(FederatedIdentity::Subject)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_federated_identity_user_provider_unique")
                    .table(FederatedIdentity::Table)
                    .col(FederatedIdentity::UserId)
                    .col(FederatedIdentity::ProviderName)
                    .unique()
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
                    .col(uuid_null(OidcLoginAttempt::LinkingUserId))
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
                    .col(string_null(TenantInvitation::TokenHash))
                    .col(uuid_null(TenantInvitation::LatestEmailId))
                    .to_owned(),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tenant_invitation_token_hash_unique")
                    .table(TenantInvitation::Table)
                    .col(TenantInvitation::TokenHash)
                    .unique()
                    .to_owned(),
            )
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
            .drop_table(Table::drop().table(FederatedIdentity::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum FederatedIdentity {
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
    LinkingUserId,
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
    TokenHash,
    LatestEmailId,
}
