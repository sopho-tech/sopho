use crate::cell::constants::CellType;
use crate::cell::dto::CellContent;
use crate::common::AppState;
use crate::cell::repository;
use crate::cell::dto;
use crate::cell::constants::CellStatus;
use sea_orm_migration::seaql_migrations::Model;
use sqlx::Connection;
use uuid::Uuid;
use axum::response::IntoResponse;
use axum::http::StatusCode;
use chrono::Utc;
use crate::entity;
use crate::notebook::does_notebook_exist;
use crate::common::errors::SophoError;
use crate::connection::service as connection_service;
use sqlx::postgres::PgConnection;
use sqlx::Row;
use sqlx::Column;
use crate::common::database_utils;
use chrono::{DateTime, FixedOffset};
use sqlx::types::JsonValue;


pub async fn does_cell_exist(
    app_state: &AppState,
    id: Uuid,
) -> bool {
    repository::get_cell(&app_state.database_connection, id).await.is_ok()
}

pub async fn get_cell(
    app_state: AppState,
    id: Uuid,
) -> impl IntoResponse {
    let cell = execute_get_cell(&app_state, id).await;
    match cell {
        Ok(cell) => {
            let response_dto = dto::CellDto::from(cell);
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
        },
        Err(e) => {
            match e {
                sea_orm::DbErr::RecordNotFound(_) => {
                    (StatusCode::NOT_FOUND, axum::Json(serde_json::json!({ "error": "Cell not found" })))
                },
                _ => {
                    (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })))
                }
            }
        }
    }
}

async fn execute_get_cell(
    app_state: &AppState,
    id: Uuid,
) -> Result<entity::cell::Model, sea_orm::DbErr> {
    return repository::get_cell(&app_state.database_connection, id).await;
}

pub async fn get_cells_by_notebook_id(
    app_state: &AppState,
    notebook_id: Uuid,
) -> Result<Vec<dto::CellDto>, SophoError> {
    let cells = repository::get_cells_by_notebook_id(&app_state.database_connection, notebook_id).await;
    match cells {
        Ok(cells) => {
            let response_dtos: Vec<dto::CellDto> = cells.into_iter().map(|cell| dto::CellDto::from(cell)).collect();
            Ok(response_dtos)
        },
        Err(e) => {
            Err(SophoError::DatabaseError(e))
        }
    }
}

pub async fn get_number_of_cells_in_notebook(app_state: &AppState, id: Uuid) -> Result<i32, SophoError> {
    let cells = get_cells_by_notebook_id(&app_state, id).await;
    match cells {
        Ok(cells) => {
            Ok(cells.len() as i32)
        },
        Err(e) => {
            Err(e)
        }
    }
}

pub async fn create_cell(
    app_state: AppState,
    payload: dto::CreateCellDto,
) -> impl IntoResponse {
    let display_order ;
    let number_of_cells = get_number_of_cells_in_notebook(&app_state, payload.notebook_id).await;
    match number_of_cells {
        Ok(number_of_cells) => {
            display_order = number_of_cells + 1;
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })));
        }
    }

    if !does_notebook_exist(&app_state, payload.notebook_id).await {
        return (StatusCode::BAD_REQUEST, axum::Json(serde_json::json!({ "error": "Notebook not found" })));
    }

    let cell_entity = entity::cell::Model {
        id: Uuid::new_v4(),
        name: payload.name,
        content: payload.content,
        cell_type: payload.cell_type.to_string(),
        status: CellStatus::Active.to_string(),
        notebook_id: payload.notebook_id,
        connection_id: payload.connection_id,
        display_order: display_order,
        created_at: Utc::now().into(),
        updated_at: Utc::now().into()
    };

    match repository::save_cell(&app_state.database_connection, cell_entity).await {
        Ok(cell) => {
            let response_dto = dto::CellDto::from(cell);
            (StatusCode::CREATED, axum::Json(serde_json::json!(response_dto)))
        },
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}


pub async fn update_cell(
    app_state: AppState,
    id: Uuid,
    payload: dto::CellDto,
) -> impl IntoResponse {
    let cell = repository::update_cell(&app_state.database_connection, id, payload).await;
    match cell {
        Ok(cell) => {
            let response_dto = dto::CellDto::from(cell);
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
        },
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

pub async fn execute_cell(
    app_state: AppState,
    id: Uuid,
) -> impl IntoResponse {
    let cell = execute_get_cell(&app_state, id).await;
    let cell = match cell {
        Ok(cell) => cell,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })));
        }
    };
    let cell_type = match CellType::from_str(&cell.cell_type)  {
        Ok(cell_type) => cell_type,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })))
    };
    match cell_type {
        CellType::Sql => return execute_sql_cell(app_state, cell).await,
        CellType::Chart => return execute_chart_cell(app_state, cell).await,
        _ => {return (StatusCode::EXPECTATION_FAILED, axum::Json(serde_json::json!({ "error": "cell type not supported" })));}
    }
}


async fn execute_sql_cell(
    app_state: AppState,
    cell: entity::cell::Model,
) -> (http::StatusCode, axum::Json<JsonValue>) {
    let connection_id = match cell.connection_id {
        Some(connection_id) => connection_id,
        None => {
            return (StatusCode::PRECONDITION_FAILED, axum::Json(serde_json::json!({ "error": "Cell has no connection assigned" })));
        }
    };
    let cell_content = match cell.content {
        Some(cell_content) => cell_content,
        None => {
            return (StatusCode::PRECONDITION_FAILED, axum::Json(serde_json::json!({ "error": "Cell has no content" })));
        }
    };

    let connection = connection_service::execute_get_connection(app_state, connection_id).await;
    let connection = match connection {
        Ok(connection) => connection,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })));
        }
    };
    
    let database_url = database_utils::get_database_url(&connection);
    let database_connection = PgConnection::connect(&database_url).await;
    let mut database_connection = match database_connection {
        Ok(database_connection) => database_connection,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, axum::Json(serde_json::json!({ "error": e.to_string() })));
        }
    };
    let result =  sqlx::query(&cell_content).fetch_all(&mut database_connection).await;
    let rows = match result {
        Ok(rows) => rows,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, axum::Json(serde_json::json!({ "error": e.to_string() })));
        }
    };
    
    let mut column_names: Vec<String> = Vec::new();
    if let Some(first_row) = rows.get(0) {
        column_names = first_row.columns().iter().map(|col| col.name().to_string()).collect();
    }

    let mut json_rows: Vec<serde_json::Value> = Vec::new();
    for row in rows {
        let mut map = serde_json::Map::new();
        for (i, col) in row.columns().iter().enumerate() {
            let value: Result<serde_json::Value, sqlx::Error> = match col.type_info().to_string().as_str() {
                "BOOL" => {
                    let value = row.try_get::<bool, _>(i);
                    value.map(serde_json::Value::Bool)
                },
                "UUID" => {
                    row.try_get::<Uuid, _>(i).map(|v| serde_json::json!(v))
                },
                "TEXT" => {
                    let value = row.try_get::<String, _>(i);
                    value.map(serde_json::Value::String)
                },
                "VARCHAR" => {
                    let value = row.try_get::<String, _>(i);
                    value.map(serde_json::Value::String)
                },
                "TIMESTAMP" => {
                    row.try_get::<DateTime<FixedOffset>, _>(i).map(|v| serde_json::json!(v))
                }
                "TIMESTAMPTZ" => {
                    row.try_get::<DateTime<FixedOffset>, _>(i).map(|v| serde_json::json!(v))
                },
                "INT4" => {
                    row.try_get::<i32, _>(i).map(|v| serde_json::json!(v))
                },
                _ => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({
                        "error": format!("data type '{}' not handled", col.type_info().to_string())
                    })));
                }
            };
            match value {
                Ok(value) => {
                    map.insert(col.name().to_string(), value);
                },
                Err(err) => {
                    if let sqlx::Error::ColumnDecode { index: _index, source: _source } = &err {
                        map.insert(col.name().to_string(), serde_json::Value::Null);
                    }
                    else {
                        return (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": err.to_string() })));
                    }
                }
            }
        }
        json_rows.push(serde_json::Value::Object(map));
    }
    return (StatusCode::OK, axum::Json(serde_json::json!(
        {
            "column_names": column_names,
            "data": json_rows,
        }
    )));
}


async fn execute_chart_cell(
    app_state: AppState,
    cell: entity::cell::Model,
) -> (http::StatusCode, axum::Json<JsonValue>) {
    let content = match cell.content {
        Some(content) => content,
        None => {return (StatusCode::PRECONDITION_FAILED, axum::Json(serde_json::json!({ "error": "Cell has no content" })))}
    };
    let cell_content =  CellContent::parse(&content, &CellType::Chart);
    let cell_content = match cell_content {
        Ok(cell_content) => cell_content,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": "Cell has non serializable content" }))
            )
        },
    };
    let cell_content = match cell_content {
        CellContent::Chart(cell_content) => cell_content,
        _ => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": "Cell has non serializable content" }))
        )
    };
    let cell_id = match cell_content.cell_id {
        Some(cell_id) => cell_id,
        None => {return (StatusCode::PRECONDITION_FAILED, axum::Json(serde_json::json!({ "error": "ChartCell has no cell id attached" })))}
    };
    let cell = execute_get_cell(&app_state, cell_id).await;
    let cell = match cell {
        Ok(cell) => cell,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })));
        }
    };
    return execute_sql_cell(app_state, cell).await;
}
