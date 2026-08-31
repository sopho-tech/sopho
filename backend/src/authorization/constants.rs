use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr, VariantArray};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    IntoStaticStr,
    VariantArray,
)]
pub enum Permission {
    #[strum(serialize = "canvas:create")]
    CanvasCreate,
    #[strum(serialize = "canvas:read")]
    CanvasRead,
    #[strum(serialize = "canvas:write")]
    CanvasWrite,
    #[strum(serialize = "canvas:delete")]
    CanvasDelete,
    #[strum(serialize = "canvas:share")]
    CanvasShare,
    #[strum(serialize = "canvas:read_private")]
    CanvasReadPrivate,
    #[strum(serialize = "canvas:write_all")]
    CanvasWriteAll,
    #[strum(serialize = "conversation:create")]
    ConversationCreate,
    #[strum(serialize = "conversation:read")]
    ConversationRead,
    #[strum(serialize = "conversation:write")]
    ConversationWrite,
    #[strum(serialize = "conversation:delete")]
    ConversationDelete,
    #[strum(serialize = "conversation:share")]
    ConversationShare,
    #[strum(serialize = "conversation:read_private")]
    ConversationReadPrivate,
    #[strum(serialize = "conversation:write_all")]
    ConversationWriteAll,
    #[strum(serialize = "connection:create")]
    ConnectionCreate,
    #[strum(serialize = "connection:read")]
    ConnectionRead,
    #[strum(serialize = "connection:write")]
    ConnectionWrite,
    #[strum(serialize = "connection:delete")]
    ConnectionDelete,
    #[strum(serialize = "connection:share")]
    ConnectionShare,
    #[strum(serialize = "connection:read_private")]
    ConnectionReadPrivate,
    #[strum(serialize = "connection:write_all")]
    ConnectionWriteAll,
    #[strum(serialize = "member:invite")]
    MemberInvite,
    #[strum(serialize = "member:remove")]
    MemberRemove,
    #[strum(serialize = "member:update_role")]
    MemberUpdateRole,
    #[strum(serialize = "role:create")]
    RoleCreate,
    #[strum(serialize = "role:update")]
    RoleUpdate,
    #[strum(serialize = "role:delete")]
    RoleDelete,
    #[strum(serialize = "tenant:update")]
    TenantUpdate,
    #[strum(serialize = "tenant:delete")]
    TenantDelete,
    #[strum(serialize = "ai_configuration:read")]
    AiConfigurationRead,
    #[strum(serialize = "ai_configuration:write")]
    AiConfigurationWrite,
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    pub fn from_str(value: &str) -> Result<Self, String> {
        <Self as std::str::FromStr>::from_str(value)
            .map_err(|_| format!("Invalid permission: {}", value))
    }

    pub fn all() -> Vec<Permission> {
        <Self as VariantArray>::VARIANTS.to_vec()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum SystemRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

impl SystemRole {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    pub fn from_str(value: &str) -> Result<Self, String> {
        <Self as std::str::FromStr>::from_str(value)
            .map_err(|_| format!("Invalid system role: {}", value))
    }

    pub fn all() -> Vec<SystemRole> {
        <Self as VariantArray>::VARIANTS.to_vec()
    }

    pub fn permissions(&self) -> Vec<Permission> {
        match self {
            SystemRole::Owner => Permission::all(),
            SystemRole::Admin => Permission::all()
                .into_iter()
                .filter(|permission| *permission != Permission::TenantDelete)
                .collect(),
            SystemRole::Member => vec![
                Permission::CanvasCreate,
                Permission::CanvasRead,
                Permission::CanvasWrite,
                Permission::CanvasDelete,
                Permission::CanvasShare,
                Permission::ConversationCreate,
                Permission::ConversationRead,
                Permission::ConversationWrite,
                Permission::ConversationDelete,
                Permission::ConversationShare,
                Permission::ConnectionRead,
                Permission::AiConfigurationRead,
            ],
            SystemRole::Viewer => vec![
                Permission::CanvasRead,
                Permission::ConversationRead,
                Permission::ConnectionRead,
                Permission::AiConfigurationRead,
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MembershipStatus {
    Active,
    Suspended,
}

impl MembershipStatus {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    pub fn from_str(value: &str) -> Result<Self, String> {
        <Self as std::str::FromStr>::from_str(value)
            .map_err(|_| format!("Invalid membership status: {}", value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantStatus {
    Active,
    Inactive,
}

impl TenantStatus {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    pub fn from_str(value: &str) -> Result<Self, String> {
        <Self as std::str::FromStr>::from_str(value)
            .map_err(|_| format!("Invalid tenant status: {}", value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum Visibility {
    Private,
    Tenant,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    pub fn from_str(value: &str) -> Result<Self, String> {
        <Self as std::str::FromStr>::from_str(value)
            .map_err(|_| format!("Invalid visibility: {}", value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ResourceType {
    Canvas,
    Conversation,
    Connection,
}

impl ResourceType {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    pub fn from_str(value: &str) -> Result<Self, String> {
        <Self as std::str::FromStr>::from_str(value)
            .map_err(|_| format!("Invalid resource type: {}", value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum PrincipalType {
    User,
}

impl PrincipalType {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AccessLevel {
    Viewer,
    Editor,
}

impl AccessLevel {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    pub fn from_str(value: &str) -> Result<Self, String> {
        <Self as std::str::FromStr>::from_str(value)
            .map_err(|_| format!("Invalid access level: {}", value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Read,
    Write,
    Delete,
    Share,
}

impl ResourceType {
    pub fn permission_for(&self, action: Action) -> Permission {
        match (self, action) {
            (ResourceType::Canvas, Action::Read) => Permission::CanvasRead,
            (ResourceType::Canvas, Action::Write) => Permission::CanvasWrite,
            (ResourceType::Canvas, Action::Delete) => Permission::CanvasDelete,
            (ResourceType::Canvas, Action::Share) => Permission::CanvasShare,
            (ResourceType::Conversation, Action::Read) => Permission::ConversationRead,
            (ResourceType::Conversation, Action::Write) => Permission::ConversationWrite,
            (ResourceType::Conversation, Action::Delete) => Permission::ConversationDelete,
            (ResourceType::Conversation, Action::Share) => Permission::ConversationShare,
            (ResourceType::Connection, Action::Read) => Permission::ConnectionRead,
            (ResourceType::Connection, Action::Write) => Permission::ConnectionWrite,
            (ResourceType::Connection, Action::Delete) => Permission::ConnectionDelete,
            (ResourceType::Connection, Action::Share) => Permission::ConnectionShare,
        }
    }

    pub fn read_private_permission(&self) -> Permission {
        match self {
            ResourceType::Canvas => Permission::CanvasReadPrivate,
            ResourceType::Conversation => Permission::ConversationReadPrivate,
            ResourceType::Connection => Permission::ConnectionReadPrivate,
        }
    }

    pub fn write_all_permission(&self) -> Permission {
        match self {
            ResourceType::Canvas => Permission::CanvasWriteAll,
            ResourceType::Conversation => Permission::ConversationWriteAll,
            ResourceType::Connection => Permission::ConnectionWriteAll,
        }
    }
}
