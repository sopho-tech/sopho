use indoc::indoc;

use crate::ai::dto::{CanvasCandidate, CanvasCandidateCell, ConversationHistory};

const GENERATE: &str = indoc! {r#"
    Design a Canvas from the conversation so far.

    ### CONVERSATION HISTORY
    {conversation_history}

    ### EXISTING CANVASES IN THIS CONVERSATION
    {existing_canvases}
"#};

const NO_EXISTING_CANVASES: &str = indoc! {r#"
    None.

    You must create a new canvas. Leave target_canvas_index null and use only create actions.
"#};

const CANVAS_SELECTION: &str = indoc! {r#"
    Set target_canvas_index to the number of the canvas to extend, or null for a new canvas.
    For update and delete, set target_cell_index to the cell number within that canvas.
"#};

pub enum UserPrompt<'a> {
    Generate {
        conversation_history: &'a ConversationHistory,
        candidates: &'a [CanvasCandidate],
    },
}

impl<'a> UserPrompt<'a> {
    pub fn render(&self) -> String {
        match self {
            UserPrompt::Generate {
                conversation_history,
                candidates,
            } => GENERATE
                .replace(
                    "{conversation_history}",
                    &format_history(conversation_history),
                )
                .replace("{existing_canvases}", &format_candidates(candidates)),
        }
    }
}

fn format_history(conversation_history: &ConversationHistory) -> String {
    conversation_history
        .into_iter()
        .enumerate()
        .map(|(index, turn)| {
            let mut turn_block = format!("Turn {}:\n  User: {}", index + 1, turn.user_question);
            if let Some(sql) = &turn.generated_sql {
                turn_block.push_str(&format!("\n  SQL: {}", sql));
            }
            turn_block
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn format_candidates(candidates: &[CanvasCandidate]) -> String {
    if candidates.is_empty() {
        return NO_EXISTING_CANVASES.to_string();
    }
    let canvases = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| format_candidate(index + 1, candidate))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("{}\n\n{}", canvases, CANVAS_SELECTION)
}

fn format_candidate(number: usize, candidate: &CanvasCandidate) -> String {
    let mut block = format!("Canvas {}: \"{}\"", number, candidate.name);
    if let Some(description) = &candidate.description {
        block.push_str(&format!("\n  {}", description));
    }
    for (index, cell) in candidate.cells.iter().enumerate() {
        block.push_str(&format!("\n{}", format_cell(index + 1, cell)));
    }
    block
}

fn format_cell(number: usize, cell: &CanvasCandidateCell) -> String {
    let chart = match &cell.chart_type {
        Some(chart_type) => chart_type.as_str(),
        None => "no chart",
    };
    format!("  Cell {}: {} [{}] {}", number, cell.title, chart, cell.sql)
}
