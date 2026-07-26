use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConversationError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("Conversion error: {0}")]
    Conversion(String),
    #[error("Conversation not found")]
    NotFound,
    #[error("{0}")]
    InvalidRequest(String),
    #[error("AI provider is not configured or not live")]
    AiNotLive,
}

#[derive(Debug, Error)]
pub enum ExecuteCompletionError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("Connection not found")]
    ConnectionNotFound,
    #[error("Conversation not found")]
    ConversationNotFound,
    #[error("There are no questions to answer")]
    NoQuestionsToAnswer,
    #[error("The last sender is not human")]
    LastSenderNotHuman,
    #[error("The last message content is invalid")]
    InvalidLastMessageContent,
    #[error("Question must not be empty")]
    EmptyQuestion,
    #[error("Question exceeds maximum length")]
    QuestionTooLong,
    #[error("AI provider is not configured or not live")]
    AiNotLive,
}

#[derive(Debug, Error)]
pub enum AppendUserMessageError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("Conversation not found")]
    NotFound,
    #[error("Conversation is busy; wait for the current response to finish")]
    ConversationBusy,
    #[error("Question must not be empty")]
    EmptyQuestion,
    #[error("Question exceeds maximum length")]
    QuestionTooLong,
    #[error("AI provider is not configured or not live")]
    AiNotLive,
}

impl From<ConversationError> for AppendUserMessageError {
    fn from(e: ConversationError) -> Self {
        match e {
            ConversationError::Database(d) => AppendUserMessageError::Database(d),
            ConversationError::Conversion(s) => {
                AppendUserMessageError::Database(sea_orm::DbErr::Custom(s))
            }
            ConversationError::NotFound => AppendUserMessageError::NotFound,
            ConversationError::InvalidRequest(s) => {
                AppendUserMessageError::Database(sea_orm::DbErr::Custom(s))
            }
            ConversationError::AiNotLive => AppendUserMessageError::AiNotLive,
        }
    }
}

impl From<ConversationError> for ExecuteCompletionError {
    fn from(e: ConversationError) -> Self {
        match e {
            ConversationError::Database(d) => ExecuteCompletionError::Database(d),
            ConversationError::Conversion(s) => {
                ExecuteCompletionError::Database(sea_orm::DbErr::Custom(s))
            }
            ConversationError::NotFound => ExecuteCompletionError::ConversationNotFound,
            ConversationError::InvalidRequest(s) => {
                ExecuteCompletionError::Database(sea_orm::DbErr::Custom(s))
            }
            ConversationError::AiNotLive => ExecuteCompletionError::AiNotLive,
        }
    }
}
