use crate::ai_usage::constants::{NO_ALLOWANCE_USAGE_SORT_NUMBER, SortDirection, UsageSortColumn};
use crate::ai_usage::datatypes::{ListedUsageRow, UsageListing, UsageSortValue};
use crate::ai_usage::dto::{
    PlatformUsageConversationDto, PlatformUsageTenantDto, PlatformUsageUserDto,
    UsageConversationDto, UsageUserDto,
};
use crate::ai_usage::error::AiUsageError;
use crate::common::constants::FIRST_PAGE;
use crate::common::server_utils::PaginatedResponse;

fn sortable_text(displayed_text: &str) -> UsageSortValue {
    UsageSortValue::Text {
        lowercase_text: displayed_text.to_lowercase(),
    }
}

fn sortable_number(number: i64) -> UsageSortValue {
    UsageSortValue::Number { number }
}

impl ListedUsageRow for UsageUserDto {
    const SORT_COLUMNS: &'static [UsageSortColumn] = &[
        UsageSortColumn::Name,
        UsageSortColumn::TotalTokens,
        UsageSortColumn::CallCount,
    ];

    fn searchable_texts(&self) -> Vec<&str> {
        vec![&self.name]
    }

    fn sort_value(&self, sort_column: UsageSortColumn) -> Option<UsageSortValue> {
        match sort_column {
            UsageSortColumn::Name => Some(sortable_text(&self.name)),
            UsageSortColumn::TotalTokens => Some(sortable_number(self.total_tokens)),
            UsageSortColumn::CallCount => Some(sortable_number(self.call_count)),
            UsageSortColumn::TenantName
            | UsageSortColumn::UserName
            | UsageSortColumn::AllowanceUsage => None,
        }
    }

    fn total_tokens(&self) -> i64 {
        self.total_tokens
    }

    fn stable_row_id(&self) -> String {
        match self.user_id {
            Some(user_id) => user_id.to_string(),
            None => self.name.clone(),
        }
    }
}

impl ListedUsageRow for UsageConversationDto {
    const SORT_COLUMNS: &'static [UsageSortColumn] = &[
        UsageSortColumn::Name,
        UsageSortColumn::UserName,
        UsageSortColumn::TotalTokens,
        UsageSortColumn::CallCount,
    ];

    fn searchable_texts(&self) -> Vec<&str> {
        vec![&self.name, &self.user_name]
    }

    fn sort_value(&self, sort_column: UsageSortColumn) -> Option<UsageSortValue> {
        match sort_column {
            UsageSortColumn::Name => Some(sortable_text(&self.name)),
            UsageSortColumn::UserName => Some(sortable_text(&self.user_name)),
            UsageSortColumn::TotalTokens => Some(sortable_number(self.total_tokens)),
            UsageSortColumn::CallCount => Some(sortable_number(self.call_count)),
            UsageSortColumn::TenantName | UsageSortColumn::AllowanceUsage => None,
        }
    }

    fn total_tokens(&self) -> i64 {
        self.total_tokens
    }

    fn stable_row_id(&self) -> String {
        self.conversation_id.to_string()
    }
}

impl ListedUsageRow for PlatformUsageTenantDto {
    const SORT_COLUMNS: &'static [UsageSortColumn] = &[
        UsageSortColumn::TenantName,
        UsageSortColumn::TotalTokens,
        UsageSortColumn::CallCount,
        UsageSortColumn::AllowanceUsage,
    ];

    fn searchable_texts(&self) -> Vec<&str> {
        vec![&self.tenant_name]
    }

    fn sort_value(&self, sort_column: UsageSortColumn) -> Option<UsageSortValue> {
        match sort_column {
            UsageSortColumn::TenantName => Some(sortable_text(&self.tenant_name)),
            UsageSortColumn::TotalTokens => Some(sortable_number(self.total_tokens)),
            UsageSortColumn::CallCount => Some(sortable_number(self.call_count)),
            UsageSortColumn::AllowanceUsage => Some(sortable_number(match &self.allowance_usage {
                Some(allowance_usage) => {
                    i64::try_from(allowance_usage.used_percentage).unwrap_or(i64::MAX)
                }
                None => NO_ALLOWANCE_USAGE_SORT_NUMBER,
            })),
            UsageSortColumn::Name | UsageSortColumn::UserName => None,
        }
    }

    fn total_tokens(&self) -> i64 {
        self.total_tokens
    }

    fn stable_row_id(&self) -> String {
        self.tenant_id.to_string()
    }
}

impl ListedUsageRow for PlatformUsageUserDto {
    const SORT_COLUMNS: &'static [UsageSortColumn] = &[
        UsageSortColumn::Name,
        UsageSortColumn::TenantName,
        UsageSortColumn::TotalTokens,
        UsageSortColumn::CallCount,
    ];

    fn searchable_texts(&self) -> Vec<&str> {
        vec![&self.name, &self.tenant_name]
    }

    fn sort_value(&self, sort_column: UsageSortColumn) -> Option<UsageSortValue> {
        match sort_column {
            UsageSortColumn::Name => Some(sortable_text(&self.name)),
            UsageSortColumn::TenantName => Some(sortable_text(&self.tenant_name)),
            UsageSortColumn::TotalTokens => Some(sortable_number(self.total_tokens)),
            UsageSortColumn::CallCount => Some(sortable_number(self.call_count)),
            UsageSortColumn::UserName | UsageSortColumn::AllowanceUsage => None,
        }
    }

    fn total_tokens(&self) -> i64 {
        self.total_tokens
    }

    fn stable_row_id(&self) -> String {
        match self.user_id {
            Some(user_id) => format!("{}{user_id}", self.tenant_id),
            None => self.tenant_id.to_string(),
        }
    }
}

impl ListedUsageRow for PlatformUsageConversationDto {
    const SORT_COLUMNS: &'static [UsageSortColumn] = &[
        UsageSortColumn::Name,
        UsageSortColumn::TenantName,
        UsageSortColumn::UserName,
        UsageSortColumn::TotalTokens,
        UsageSortColumn::CallCount,
    ];

    fn searchable_texts(&self) -> Vec<&str> {
        vec![&self.name, &self.tenant_name, &self.user_name]
    }

    fn sort_value(&self, sort_column: UsageSortColumn) -> Option<UsageSortValue> {
        match sort_column {
            UsageSortColumn::Name => Some(sortable_text(&self.name)),
            UsageSortColumn::TenantName => Some(sortable_text(&self.tenant_name)),
            UsageSortColumn::UserName => Some(sortable_text(&self.user_name)),
            UsageSortColumn::TotalTokens => Some(sortable_number(self.total_tokens)),
            UsageSortColumn::CallCount => Some(sortable_number(self.call_count)),
            UsageSortColumn::AllowanceUsage => None,
        }
    }

    fn total_tokens(&self) -> i64 {
        self.total_tokens
    }

    fn stable_row_id(&self) -> String {
        self.conversation_id.to_string()
    }
}

struct SortableUsageRow<UsageRow> {
    sort_value: UsageSortValue,
    usage_row: UsageRow,
}

pub(super) fn list_usage_rows<UsageRow: ListedUsageRow>(
    usage_rows: Vec<UsageRow>,
    usage_listing: &UsageListing,
) -> Result<PaginatedResponse<UsageRow>, AiUsageError> {
    let usage_ordering = &usage_listing.usage_ordering;
    if !UsageRow::SORT_COLUMNS.contains(&usage_ordering.sort_column) {
        return Err(AiUsageError::SortColumnNotAllowed);
    }
    let mut sortable_usage_rows = Vec::with_capacity(usage_rows.len());
    for usage_row in usage_rows {
        let is_matching_the_search =
            usage_ordering
                .lowercase_search_text
                .as_ref()
                .is_none_or(|lowercase_search_text| {
                    usage_row.searchable_texts().iter().any(|searchable_text| {
                        searchable_text
                            .to_lowercase()
                            .contains(lowercase_search_text)
                    })
                });
        if !is_matching_the_search {
            continue;
        }
        sortable_usage_rows.push(SortableUsageRow {
            sort_value: usage_row
                .sort_value(usage_ordering.sort_column)
                .ok_or(AiUsageError::SortColumnNotAllowed)?,
            usage_row,
        });
    }
    sortable_usage_rows.sort_by(|first_row, second_row| {
        let ascending_ordering = first_row.sort_value.cmp(&second_row.sort_value);
        let requested_ordering = match usage_ordering.sort_direction {
            SortDirection::Ascending => ascending_ordering,
            SortDirection::Descending => ascending_ordering.reverse(),
        };
        requested_ordering
            .then_with(|| {
                second_row
                    .usage_row
                    .total_tokens()
                    .cmp(&first_row.usage_row.total_tokens())
            })
            .then_with(|| {
                first_row
                    .usage_row
                    .stable_row_id()
                    .cmp(&second_row.usage_row.stable_row_id())
            })
    });

    let usage_page = &usage_listing.usage_page;
    let total_items = sortable_usage_rows.len() as u64;
    let total_pages = total_items.div_ceil(usage_page.page_size);
    if usage_page.page != FIRST_PAGE && usage_page.page >= total_pages {
        return Err(AiUsageError::PageOutOfRange);
    }
    let first_row_offset =
        usize::try_from(usage_page.first_row_offset).map_err(|_| AiUsageError::PageOutOfRange)?;
    let page_size =
        usize::try_from(usage_page.page_size).map_err(|_| AiUsageError::PageOutOfRange)?;
    Ok(PaginatedResponse {
        data: sortable_usage_rows
            .into_iter()
            .skip(first_row_offset)
            .take(page_size)
            .map(|sortable_usage_row| sortable_usage_row.usage_row)
            .collect(),
        total_pages,
        page: usage_page.page,
        page_size: usage_page.page_size,
        total_items,
    })
}
