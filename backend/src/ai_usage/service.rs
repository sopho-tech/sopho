use crate::ai::agent_utils::AgentName;
use crate::ai_usage::constants::{
    AiUsageKind, BACKGROUND_WORK_LABEL, CACHED_INPUT_TOKEN_KIND, CANVAS_COMMAND_LABEL,
    ConversationNameKind, DEFAULT_CONVERSATION_PAGE_SIZE, DEFAULT_RANGE_DAY_COUNT,
    DELETED_ACCOUNT_LABEL, DELETED_CONVERSATION_LABEL, DELETED_QUESTION_LABEL, INPUT_TOKEN_KIND,
    MAXIMUM_CONVERSATION_PAGE_SIZE, MAXIMUM_RANGE_DAY_COUNT, NO_QUESTION_LABEL,
    ONE_CALL_WITHOUT_TOKEN_COUNTS_NOTE, OUTPUT_TOKEN_KIND, PRIVATE_CONVERSATION_LABEL,
    PRIVATE_QUESTION_LABEL, QuestionTextKind, UTC_DAY_FORMAT,
};
use crate::ai_usage::dto::{
    AiCallContext, ConversationPageQuery, ModelCallRecord, UsageByKindDto, UsageByUserDto,
    UsageConversationDto, UsageConversationQuestionsDto, UsageDayDto, UsageKindDto,
    UsageOverTimeDto, UsageQuestionDto, UsageRange, UsageRangeQuery, UsageSummaryDto, UsageUserDto,
};
use crate::ai_usage::error::AiUsageError;
use crate::ai_usage::repository;
use crate::authentication::service as authentication_service;
use crate::authorization::constants::Permission;
use crate::authorization::dto::TenantContext;
use crate::authorization::service::require_capability;
use crate::common::AppState;
use crate::common::server_utils::PaginatedResponse;
use crate::common::time_utils;
use crate::conversational_analytics::conversation::dto::{
    ConversationQuestionsForUsage, QuestionForUsage,
};
use crate::conversational_analytics::conversation::service as conversation_service;
use crate::entity::ai_usage;
use chrono::{Days, NaiveDate, NaiveTime};
use sea_orm::Set;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

pub async fn record_model_call(
    app_state: &AppState,
    call_context: &AiCallContext,
    model_call_record: ModelCallRecord,
) -> Result<(), AiUsageError> {
    let token_usage = model_call_record.token_usage;
    let key_source: &'static str = model_call_record.key_source.into();
    let ai_usage_row = ai_usage::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(call_context.tenant_id().get_uuid()),
        user_id: Set(call_context.user_id().map(|user_id| user_id.get_uuid())),
        agent_name: Set(model_call_record.agent_name.to_string()),
        provider: Set(model_call_record.provider.to_string()),
        model: Set(model_call_record.model.to_string()),
        key_source: Set(key_source.to_string()),
        input_tokens: Set(storable_token_count(
            INPUT_TOKEN_KIND,
            token_usage.input_tokens,
        )?),
        output_tokens: Set(storable_token_count(
            OUTPUT_TOKEN_KIND,
            token_usage.output_tokens,
        )?),
        cached_input_tokens: Set(storable_token_count(
            CACHED_INPUT_TOKEN_KIND,
            token_usage.cached_input_tokens,
        )?),
        metadata: Set(serde_json::to_value(call_context.metadata())?),
        created_at: Set(time_utils::now_utc_into()),
    };
    repository::insert_ai_usage(&app_state.database_connection, ai_usage_row).await?;
    Ok(())
}

fn storable_token_count(token_kind: &'static str, token_count: u64) -> Result<i64, AiUsageError> {
    i64::try_from(token_count).map_err(|_| AiUsageError::TokenCountOutOfRange {
        token_kind,
        token_count,
    })
}

pub fn kind_of_agent(agent_name: AgentName) -> AiUsageKind {
    match agent_name {
        AgentName::RouterAgent
        | AgentName::HypothesisGenerationAgent
        | AgentName::IntegrateCandidatePlanAgent
        | AgentName::DataCatalogDeletionAgent
        | AgentName::DataCatalogSelectionAgent
        | AgentName::FunctionalRoleAnalysisAgent
        | AgentName::DataProfilingBeforeAgent
        | AgentName::SchemaLinkingFinalSynthesisAgent
        | AgentName::SqlGenerationAgent
        | AgentName::ResultNarrationAgent
        | AgentName::VisualizationRecommendationAgent => AiUsageKind::AnsweringQuestions,
        AgentName::FollowupQuestionsAgent => AiUsageKind::FollowupQuestions,
        AgentName::CanvasGenerationAgent => AiUsageKind::CanvasGeneration,
        AgentName::ChartSummaryAgent | AgentName::DashboardSummaryAgent => AiUsageKind::Summaries,
        AgentName::SuggestedQuestionsAgent => AiUsageKind::SuggestedQuestions,
        AgentName::ConversationNameAgent => AiUsageKind::ConversationNaming,
        AgentName::LivenessCheckAgent => AiUsageKind::AiKeyTest,
    }
}

pub fn resolve_usage_range(
    usage_range_query: &UsageRangeQuery,
    today: NaiveDate,
) -> Result<UsageRange, AiUsageError> {
    let last_day = usage_range_query.to.unwrap_or(today);
    let first_day = match (usage_range_query.from, usage_range_query.last_day_count) {
        (Some(_), Some(_)) => return Err(AiUsageError::RangeAmbiguous),
        (Some(first_day), None) => first_day,
        (None, requested_day_count) => {
            let day_count = requested_day_count.unwrap_or(DEFAULT_RANGE_DAY_COUNT);
            if day_count < 1 {
                return Err(AiUsageError::RangeWithoutDays);
            }
            last_day
                .checked_sub_days(Days::new(day_count.unsigned_abs() - 1))
                .ok_or(AiUsageError::RangeOutOfCalendar)?
        }
    };
    if first_day > last_day {
        return Err(AiUsageError::RangeReversed);
    }
    let day_count = (last_day - first_day).num_days() + 1;
    if day_count > MAXIMUM_RANGE_DAY_COUNT {
        return Err(AiUsageError::RangeTooLong {
            maximum_day_count: MAXIMUM_RANGE_DAY_COUNT,
        });
    }
    let day_after_last_day = last_day
        .checked_add_days(Days::new(1))
        .ok_or(AiUsageError::RangeOutOfCalendar)?;
    Ok(UsageRange {
        first_day,
        last_day,
        starts_at: first_day.and_time(NaiveTime::MIN).and_utc(),
        ends_before: day_after_last_day.and_time(NaiveTime::MIN).and_utc(),
    })
}

fn authorized_usage_range(
    tenant_context: &TenantContext,
    usage_range_query: &UsageRangeQuery,
) -> Result<UsageRange, AiUsageError> {
    require_capability(tenant_context, Permission::UsageRead)?;
    resolve_usage_range(usage_range_query, time_utils::now_utc().date_naive())
}

pub async fn get_usage_summary(
    app_state: &AppState,
    tenant_context: &TenantContext,
    usage_range_query: &UsageRangeQuery,
) -> Result<UsageSummaryDto, AiUsageError> {
    let usage_range = authorized_usage_range(tenant_context, usage_range_query)?;
    let tenant_id = tenant_context.tenant_id();
    let totals = repository::sum_usage(&app_state.database_connection, tenant_id, &usage_range)
        .await?
        .ok_or_else(|| missing_totals_error("the usage summary"))?;
    let calls_without_token_counts = repository::count_calls_without_token_counts(
        &app_state.database_connection,
        tenant_id,
        &usage_range,
    )
    .await?
    .ok_or_else(|| missing_totals_error("the calls without token counts"))?
    .call_count;
    let missing_token_counts_note = match calls_without_token_counts {
        0 => None,
        1 => Some(ONE_CALL_WITHOUT_TOKEN_COUNTS_NOTE.to_string()),
        _ => Some(format!(
            "{calls_without_token_counts} calls in this range reported no token counts. \
             They are included in the number of calls but add no tokens."
        )),
    };
    Ok(UsageSummaryDto {
        from: usage_range.first_day,
        to: usage_range.last_day,
        input_tokens: totals.input_tokens,
        output_tokens: totals.output_tokens,
        cached_input_tokens: totals.cached_input_tokens,
        total_tokens: totals.input_tokens + totals.output_tokens,
        call_count: totals.call_count,
        has_usage: totals.call_count > 0,
        missing_token_counts_note,
    })
}

fn missing_totals_error(queried_totals: &str) -> AiUsageError {
    AiUsageError::UnreadableStoredValue(format!("{queried_totals} returned no row"))
}

pub async fn get_usage_over_time(
    app_state: &AppState,
    tenant_context: &TenantContext,
    usage_range_query: &UsageRangeQuery,
) -> Result<UsageOverTimeDto, AiUsageError> {
    let usage_range = authorized_usage_range(tenant_context, usage_range_query)?;
    let day_rows = repository::sum_usage_by_day(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        &usage_range,
    )
    .await?;
    let mut day_rows_by_day = HashMap::new();
    for day_row in day_rows {
        let day = NaiveDate::parse_from_str(&day_row.day, UTC_DAY_FORMAT)
            .map_err(|_| AiUsageError::UnreadableStoredValue(format!("day {}", day_row.day)))?;
        day_rows_by_day.insert(day, day_row);
    }
    let days = usage_range
        .first_day
        .iter_days()
        .take_while(|day| *day <= usage_range.last_day)
        .map(|day| match day_rows_by_day.get(&day) {
            Some(day_row) => UsageDayDto {
                day,
                input_tokens: day_row.input_tokens,
                output_tokens: day_row.output_tokens,
                total_tokens: day_row.input_tokens + day_row.output_tokens,
                call_count: day_row.call_count,
            },
            None => UsageDayDto {
                day,
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 0,
                call_count: 0,
            },
        })
        .collect();
    Ok(UsageOverTimeDto { days })
}

async fn load_user_names(
    app_state: &AppState,
    user_ids: Vec<Uuid>,
) -> Result<HashMap<Uuid, String>, AiUsageError> {
    let users = authentication_service::find_users_by_ids(app_state, user_ids).await?;
    Ok(users
        .into_iter()
        .map(|user| (user.id, user.full_name))
        .collect())
}

fn user_name(user_names: &HashMap<Uuid, String>, user_id: Option<Uuid>) -> String {
    match user_id {
        None => BACKGROUND_WORK_LABEL.to_string(),
        Some(user_id) => match user_names.get(&user_id) {
            Some(full_name) => full_name.clone(),
            None => DELETED_ACCOUNT_LABEL.to_string(),
        },
    }
}

pub async fn get_usage_by_user(
    app_state: &AppState,
    tenant_context: &TenantContext,
    usage_range_query: &UsageRangeQuery,
) -> Result<UsageByUserDto, AiUsageError> {
    let usage_range = authorized_usage_range(tenant_context, usage_range_query)?;
    let user_rows = repository::sum_usage_by_user(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        &usage_range,
    )
    .await?;
    let user_names = load_user_names(
        app_state,
        user_rows
            .iter()
            .filter_map(|user_row| user_row.user_id)
            .collect(),
    )
    .await?;
    let users = user_rows
        .into_iter()
        .map(|user_row| UsageUserDto {
            user_id: user_row.user_id,
            name: user_name(&user_names, user_row.user_id),
            total_tokens: user_row.input_tokens + user_row.output_tokens,
            call_count: user_row.call_count,
        })
        .collect();
    Ok(UsageByUserDto { users })
}

pub async fn get_usage_by_kind(
    app_state: &AppState,
    tenant_context: &TenantContext,
    usage_range_query: &UsageRangeQuery,
) -> Result<UsageByKindDto, AiUsageError> {
    let usage_range = authorized_usage_range(tenant_context, usage_range_query)?;
    let agent_rows = repository::sum_usage_by_agent(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        &usage_range,
    )
    .await?;
    let mut kinds: Vec<UsageKindDto> = Vec::new();
    for agent_row in agent_rows {
        let agent_name =
            AgentName::from_str(&agent_row.agent_name).map_err(|_| AiUsageError::UnknownAgent {
                agent_name: agent_row.agent_name.clone(),
            })?;
        let usage_kind = kind_of_agent(agent_name);
        let kind_tokens = agent_row.input_tokens + agent_row.output_tokens;
        match kinds
            .iter_mut()
            .find(|kind| kind.label == usage_kind.label())
        {
            Some(kind) => {
                kind.total_tokens += kind_tokens;
                kind.call_count += agent_row.call_count;
            }
            None => kinds.push(UsageKindDto {
                kind: usage_kind.into(),
                label: usage_kind.label(),
                total_tokens: kind_tokens,
                call_count: agent_row.call_count,
            }),
        }
    }
    kinds.sort_by(|first_kind, second_kind| {
        second_kind
            .total_tokens
            .cmp(&first_kind.total_tokens)
            .then(first_kind.label.cmp(second_kind.label))
    });
    Ok(UsageByKindDto { kinds })
}

fn stored_uuid(stored_value: &str) -> Result<Uuid, AiUsageError> {
    Uuid::parse_str(stored_value)
        .map_err(|_| AiUsageError::UnreadableStoredValue(format!("id {stored_value}")))
}

struct LargestUser {
    total_tokens: i64,
    user_id: Option<Uuid>,
}

impl LargestUser {
    fn is_larger_than(&self, other_user: &LargestUser) -> bool {
        (self.total_tokens, other_user.user_id) > (other_user.total_tokens, self.user_id)
    }
}

struct ConversationName {
    name: String,
    name_kind: ConversationNameKind,
}

fn conversation_name(name_when_readable_of_existing: Option<&Option<String>>) -> ConversationName {
    match name_when_readable_of_existing {
        Some(Some(name)) => ConversationName {
            name: name.clone(),
            name_kind: ConversationNameKind::Name,
        },
        Some(None) => ConversationName {
            name: PRIVATE_CONVERSATION_LABEL.to_string(),
            name_kind: ConversationNameKind::Private,
        },
        None => ConversationName {
            name: DELETED_CONVERSATION_LABEL.to_string(),
            name_kind: ConversationNameKind::Deleted,
        },
    }
}

pub async fn list_usage_by_conversation(
    app_state: &AppState,
    tenant_context: &TenantContext,
    usage_range_query: &UsageRangeQuery,
    conversation_page_query: &ConversationPageQuery,
) -> Result<PaginatedResponse<UsageConversationDto>, AiUsageError> {
    let usage_range = authorized_usage_range(tenant_context, usage_range_query)?;
    let tenant_id = tenant_context.tenant_id();
    let page = conversation_page_query.page.unwrap_or(0);
    let page_size = conversation_page_query
        .page_size
        .unwrap_or(DEFAULT_CONVERSATION_PAGE_SIZE)
        .clamp(1, MAXIMUM_CONVERSATION_PAGE_SIZE);
    let conversation_count = repository::count_conversations_with_usage(
        &app_state.database_connection,
        tenant_id,
        &usage_range,
    )
    .await?
    .ok_or_else(|| missing_totals_error("the number of conversations with usage"))?;
    let total_items = u64::try_from(conversation_count).map_err(|_| {
        AiUsageError::UnreadableStoredValue(format!("conversation count {conversation_count}"))
    })?;
    let first_row_offset = page
        .checked_mul(page_size)
        .ok_or(AiUsageError::PageOutOfRange)?;
    let conversation_rows = repository::sum_usage_by_conversation(
        &app_state.database_connection,
        tenant_id,
        &usage_range,
        first_row_offset,
        page_size,
    )
    .await?;

    let stored_conversation_ids: Vec<String> = conversation_rows
        .iter()
        .map(|conversation_row| conversation_row.conversation_id.clone())
        .collect();
    let conversation_ids = stored_conversation_ids
        .iter()
        .map(|stored_conversation_id| stored_uuid(stored_conversation_id))
        .collect::<Result<Vec<Uuid>, AiUsageError>>()?;
    let name_when_readable_by_conversation_id: HashMap<Uuid, Option<String>> =
        conversation_service::describe_conversations_for_usage(
            app_state,
            tenant_context,
            &conversation_ids,
        )
        .await?
        .into_iter()
        .map(|description| (description.conversation_id, description.name_when_readable))
        .collect();

    let conversation_user_rows = repository::sum_usage_by_conversation_and_user(
        &app_state.database_connection,
        tenant_id,
        &usage_range,
        &stored_conversation_ids,
    )
    .await?;
    let mut largest_user_by_conversation_id: HashMap<String, LargestUser> = HashMap::new();
    for conversation_user_row in conversation_user_rows {
        let user = LargestUser {
            total_tokens: conversation_user_row.input_tokens + conversation_user_row.output_tokens,
            user_id: conversation_user_row.user_id,
        };
        let is_largest_so_far = largest_user_by_conversation_id
            .get(&conversation_user_row.conversation_id)
            .is_none_or(|largest_user| user.is_larger_than(largest_user));
        if is_largest_so_far {
            largest_user_by_conversation_id.insert(conversation_user_row.conversation_id, user);
        }
    }
    let user_names = load_user_names(
        app_state,
        largest_user_by_conversation_id
            .values()
            .filter_map(|largest_user| largest_user.user_id)
            .collect(),
    )
    .await?;

    let mut conversations = Vec::with_capacity(conversation_rows.len());
    for (conversation_row, conversation_id) in conversation_rows.into_iter().zip(conversation_ids) {
        let largest_user = largest_user_by_conversation_id
            .get(&conversation_row.conversation_id)
            .ok_or_else(|| {
                AiUsageError::UnreadableStoredValue(format!(
                    "conversation {conversation_id} has usage but no user totals"
                ))
            })?;
        let ConversationName { name, name_kind } =
            conversation_name(name_when_readable_by_conversation_id.get(&conversation_id));
        conversations.push(UsageConversationDto {
            conversation_id,
            name,
            name_kind,
            user_name: user_name(&user_names, largest_user.user_id),
            total_tokens: conversation_row.input_tokens + conversation_row.output_tokens,
            call_count: conversation_row.call_count,
        });
    }
    Ok(PaginatedResponse {
        data: conversations,
        total_pages: total_items.div_ceil(page_size),
        page,
        page_size,
        total_items,
    })
}

struct QuestionText {
    text: String,
    text_kind: QuestionTextKind,
}

fn question_text(
    conversation_questions: Option<&ConversationQuestionsForUsage>,
    conversation_message_id: Option<Uuid>,
) -> Result<QuestionText, AiUsageError> {
    let Some(conversation_message_id) = conversation_message_id else {
        return Ok(QuestionText {
            text: NO_QUESTION_LABEL.to_string(),
            text_kind: QuestionTextKind::NoQuestion,
        });
    };
    let Some(conversation_questions) = conversation_questions else {
        return Ok(QuestionText {
            text: DELETED_QUESTION_LABEL.to_string(),
            text_kind: QuestionTextKind::Deleted,
        });
    };
    if !conversation_questions.is_readable {
        return Ok(QuestionText {
            text: PRIVATE_QUESTION_LABEL.to_string(),
            text_kind: QuestionTextKind::Private,
        });
    }
    match conversation_questions
        .questions_by_answer_message_id
        .get(&conversation_message_id)
    {
        Some(QuestionForUsage::Text { text }) => Ok(QuestionText {
            text: text.clone(),
            text_kind: QuestionTextKind::Text,
        }),
        Some(QuestionForUsage::Command) => Ok(QuestionText {
            text: CANVAS_COMMAND_LABEL.to_string(),
            text_kind: QuestionTextKind::Command,
        }),
        None => Err(AiUsageError::UnreadableStoredValue(format!(
            "usage points at answer message {conversation_message_id}, which the conversation does not have"
        ))),
    }
}

pub async fn get_usage_of_conversation_questions(
    app_state: &AppState,
    tenant_context: &TenantContext,
    usage_range_query: &UsageRangeQuery,
    conversation_id: Uuid,
) -> Result<UsageConversationQuestionsDto, AiUsageError> {
    let usage_range = authorized_usage_range(tenant_context, usage_range_query)?;
    let question_rows = repository::sum_usage_by_question(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        &usage_range,
        conversation_id,
    )
    .await?;
    let conversation_questions = conversation_service::find_conversation_questions_for_usage(
        app_state,
        tenant_context,
        conversation_id,
    )
    .await?;
    let name_when_readable_by_conversation_id: HashMap<Uuid, Option<String>> =
        conversation_service::describe_conversations_for_usage(
            app_state,
            tenant_context,
            &[conversation_id],
        )
        .await?
        .into_iter()
        .map(|description| (description.conversation_id, description.name_when_readable))
        .collect();
    let ConversationName { name, name_kind } =
        conversation_name(name_when_readable_by_conversation_id.get(&conversation_id));

    let mut questions = Vec::with_capacity(question_rows.len());
    let mut total_tokens = 0;
    let mut call_count = 0;
    let mut question_count = 0;
    for question_row in question_rows {
        let conversation_message_id = question_row
            .conversation_message_id
            .as_deref()
            .map(stored_uuid)
            .transpose()?;
        let QuestionText { text, text_kind } =
            question_text(conversation_questions.as_ref(), conversation_message_id)?;
        let question_tokens = question_row.input_tokens + question_row.output_tokens;
        total_tokens += question_tokens;
        call_count += question_row.call_count;
        if conversation_message_id.is_some() {
            question_count += 1;
        }
        questions.push(UsageQuestionDto {
            conversation_message_id,
            text,
            text_kind,
            first_call_at: question_row.first_call_at,
            total_tokens: question_tokens,
            call_count: question_row.call_count,
        });
    }
    Ok(UsageConversationQuestionsDto {
        conversation_id,
        name,
        name_kind,
        total_tokens,
        call_count,
        question_count,
        questions,
    })
}

#[cfg(test)]
mod tests {
    use super::{kind_of_agent, resolve_usage_range};
    use crate::ai::agent_utils::AgentName;
    use crate::ai_usage::constants::AiUsageKind;
    use crate::ai_usage::dto::UsageRangeQuery;
    use crate::ai_usage::error::AiUsageError;
    use chrono::NaiveDate;
    use std::str::FromStr;

    const STORED_AGENT_NAMES: [&str; 18] = [
        "canvas_generation_agent",
        "chart_summary_agent",
        "conversation_name_agent",
        "dashboard_summary_agent",
        "liveness_check_agent",
        "data_catalog_deletion_agent",
        "data_catalog_selection_agent",
        "data_profiling_before_agent",
        "followup_questions_agent",
        "functional_role_analysis_agent",
        "hypothesis_generation_agent",
        "integrate_candidate_plan_agent",
        "result_narration_agent",
        "router_agent",
        "schema_linking_final_synthesis_agent",
        "sql_generation_agent",
        "suggested_questions_agent",
        "visualization_recommendation_agent",
    ];

    fn day(year: i32, month: u32, day_of_month: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day_of_month).expect("a valid day")
    }

    #[test]
    fn agents_are_grouped_into_the_kinds_of_work_people_recognise() {
        assert_eq!(
            kind_of_agent(AgentName::SqlGenerationAgent),
            AiUsageKind::AnsweringQuestions
        );
        assert_eq!(
            kind_of_agent(AgentName::DashboardSummaryAgent),
            AiUsageKind::Summaries
        );
        assert_eq!(
            kind_of_agent(AgentName::ConversationNameAgent),
            AiUsageKind::ConversationNaming
        );
        assert_eq!(
            kind_of_agent(AgentName::LivenessCheckAgent),
            AiUsageKind::AiKeyTest
        );
    }

    #[test]
    fn stored_agent_names_still_name_an_agent() {
        for stored_agent_name in STORED_AGENT_NAMES {
            assert!(
                AgentName::from_str(stored_agent_name).is_ok(),
                "usage rows hold {stored_agent_name}; renaming the agent needs a data migration"
            );
        }
    }

    #[test]
    fn a_range_with_no_days_given_is_the_last_thirty_days() {
        let usage_range = resolve_usage_range(
            &UsageRangeQuery {
                from: None,
                to: None,
                last_day_count: None,
            },
            day(2026, 11, 15),
        )
        .expect("the default range");

        assert_eq!(usage_range.first_day, day(2026, 10, 17));
        assert_eq!(usage_range.last_day, day(2026, 11, 15));
        assert_eq!(
            usage_range.starts_at.to_rfc3339(),
            "2026-10-17T00:00:00+00:00"
        );
        assert_eq!(
            usage_range.ends_before.to_rfc3339(),
            "2026-11-16T00:00:00+00:00"
        );
    }

    #[test]
    fn a_number_of_days_counts_back_from_the_last_day() {
        let usage_range = resolve_usage_range(
            &UsageRangeQuery {
                from: None,
                to: None,
                last_day_count: Some(7),
            },
            day(2026, 11, 15),
        )
        .expect("the last seven days");

        assert_eq!(usage_range.first_day, day(2026, 11, 9));
        assert_eq!(usage_range.last_day, day(2026, 11, 15));
    }

    #[test]
    fn a_number_of_days_together_with_a_first_day_is_refused() {
        let refused = resolve_usage_range(
            &UsageRangeQuery {
                from: Some(day(2026, 11, 1)),
                to: None,
                last_day_count: Some(7),
            },
            day(2026, 11, 15),
        );

        assert!(matches!(refused, Err(AiUsageError::RangeAmbiguous)));
    }

    #[test]
    fn a_single_day_is_a_valid_range() {
        let usage_range = resolve_usage_range(
            &UsageRangeQuery {
                from: Some(day(2026, 11, 14)),
                to: Some(day(2026, 11, 14)),
                last_day_count: None,
            },
            day(2026, 11, 15),
        )
        .expect("a single day");

        assert_eq!(usage_range.first_day, usage_range.last_day);
    }

    #[test]
    fn a_reversed_range_is_refused() {
        let refused = resolve_usage_range(
            &UsageRangeQuery {
                from: Some(day(2026, 11, 15)),
                to: Some(day(2026, 11, 14)),
                last_day_count: None,
            },
            day(2026, 11, 15),
        );

        assert!(matches!(refused, Err(AiUsageError::RangeReversed)));
    }

    #[test]
    fn a_range_longer_than_the_maximum_is_refused() {
        let refused = resolve_usage_range(
            &UsageRangeQuery {
                from: Some(day(2025, 11, 14)),
                to: Some(day(2026, 11, 15)),
                last_day_count: None,
            },
            day(2026, 11, 15),
        );
        let allowed = resolve_usage_range(
            &UsageRangeQuery {
                from: Some(day(2025, 11, 15)),
                to: Some(day(2026, 11, 15)),
                last_day_count: None,
            },
            day(2026, 11, 15),
        );

        assert!(matches!(
            refused,
            Err(AiUsageError::RangeTooLong {
                maximum_day_count: 366
            })
        ));
        assert!(allowed.is_ok());
    }
}
