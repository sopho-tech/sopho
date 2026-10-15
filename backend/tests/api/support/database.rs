use crate::support::TestApp;
use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::sea_query::{Alias, Asterisk, Expr, Query, SelectStatement, SimpleExpr};
use sea_orm::{ConnectionTrait, DbBackend, DbErr, QueryResult, Value};
use uuid::Uuid;

pub type Column<'a> = (&'a str, Value);

const ROW_COUNT_ALIAS: &str = "row_count";

pub fn now() -> DateTime<FixedOffset> {
    Utc::now().fixed_offset()
}

pub fn backend(app: &TestApp) -> DbBackend {
    app.app_state.database_connection.get_database_backend()
}

fn filtered(mut statement: SelectStatement, filters: &[Column<'_>]) -> SelectStatement {
    for (column, value) in filters {
        statement.and_where(Expr::col(Alias::new(*column)).eq(SimpleExpr::Value(value.clone())));
    }
    statement
}

pub async fn try_insert_row(
    app: &TestApp,
    table: &str,
    columns: &[Column<'_>],
) -> Result<(), DbErr> {
    let statement = Query::insert()
        .into_table(Alias::new(table))
        .columns(columns.iter().map(|(column, _)| Alias::new(*column)))
        .values_panic(
            columns
                .iter()
                .map(|(_, value)| SimpleExpr::Value(value.clone())),
        )
        .to_owned();
    app.app_state
        .database_connection
        .execute(backend(app).build(&statement))
        .await
        .map(|_| ())
}

pub async fn insert_row(app: &TestApp, table: &str, columns: &[Column<'_>]) {
    try_insert_row(app, table, columns)
        .await
        .unwrap_or_else(|error| panic!("insert into {table}: {error}"));
}

pub async fn update_rows(
    app: &TestApp,
    table: &str,
    changes: &[Column<'_>],
    filters: &[Column<'_>],
) -> u64 {
    let mut statement = Query::update();
    statement.table(Alias::new(table)).values(
        changes
            .iter()
            .map(|(column, value)| (Alias::new(*column), SimpleExpr::Value(value.clone()))),
    );
    for (column, value) in filters {
        statement.and_where(Expr::col(Alias::new(*column)).eq(SimpleExpr::Value(value.clone())));
    }
    app.app_state
        .database_connection
        .execute(backend(app).build(&statement))
        .await
        .unwrap_or_else(|error| panic!("update {table}: {error}"))
        .rows_affected()
}

pub async fn count_rows(app: &TestApp, table: &str, filters: &[Column<'_>]) -> i64 {
    let statement = filtered(
        Query::select()
            .expr_as(Expr::col(Asterisk).count(), Alias::new(ROW_COUNT_ALIAS))
            .from(Alias::new(table))
            .to_owned(),
        filters,
    );
    app.app_state
        .database_connection
        .query_one(backend(app).build(&statement))
        .await
        .unwrap_or_else(|error| panic!("count {table}: {error}"))
        .expect("a count returns a row")
        .try_get::<i64>("", ROW_COUNT_ALIAS)
        .expect("read the row count")
}

pub async fn select_rows(
    app: &TestApp,
    table: &str,
    columns: &[&str],
    filters: &[Column<'_>],
) -> Vec<QueryResult> {
    let statement = filtered(
        Query::select()
            .columns(columns.iter().map(|column| Alias::new(*column)))
            .from(Alias::new(table))
            .to_owned(),
        filters,
    );
    app.app_state
        .database_connection
        .query_all(backend(app).build(&statement))
        .await
        .unwrap_or_else(|error| panic!("select from {table}: {error}"))
}

pub async fn select_uuid(app: &TestApp, table: &str, column: &str, filters: &[Column<'_>]) -> Uuid {
    let rows = select_rows(app, table, &[column], filters).await;
    assert_eq!(rows.len(), 1, "expected one {table} row");
    rows[0]
        .try_get::<Uuid>("", column)
        .unwrap_or_else(|error| panic!("read {table}.{column}: {error}"))
}
