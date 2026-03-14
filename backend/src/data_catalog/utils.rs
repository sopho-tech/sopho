use super::dto::{Column, Database, Schema, Table};
use crate::{
    ai::dto::{DeletionSet, SelectionSet},
    common::errors::GetDataCatalogError,
    data_catalog::service::get_data_catalog_of_connection,
    entity,
};
use std::collections::HashMap;

pub fn join_pruned_batches(batches: Vec<Database>) -> Database {
    let (name, description) = batches
        .first()
        .map(|db| (db.name.clone(), db.description.clone()))
        .unwrap();
    let mut result = Database {
        name,
        description,
        schemas: HashMap::new(),
    };

    for database in batches {
        for (_, schema) in database.schemas {
            let columns_included: HashMap<String, Vec<(&str, &Column)>> = schema
                .tables
                .iter()
                .map(|(table_name, table)| {
                    let cols: Vec<_> = table
                        .columns
                        .iter()
                        .filter(|(_, col)| !(!col.should_select && col.should_delete))
                        .map(|(name, col)| (name.as_str(), col))
                        .collect();
                    (table_name.clone(), cols)
                })
                .collect();

            let tables_included: Vec<(&Table, Vec<&Column>)> = schema
                .tables
                .iter()
                .filter_map(|(_, table)| {
                    let cols: Vec<&Column> = columns_included
                        .get(&table.name)
                        .map(|c| c.iter().map(|(_, col)| *col).collect())
                        .unwrap_or_default();
                    let included = table.should_select || !cols.is_empty();
                    if included {
                        Some((table, cols))
                    } else {
                        None
                    }
                })
                .collect();

            let schema_included = schema.should_select || !tables_included.is_empty();
            if !schema_included {
                continue;
            }

            let result_schema = result
                .schemas
                .entry(schema.name.clone())
                .or_insert_with(|| Schema {
                    name: schema.name.clone(),
                    description: schema.description.clone(),
                    tables: HashMap::new(),
                    should_delete: false,
                    should_select: false,
                });

            for (table, columns) in tables_included {
                if columns.is_empty() && !table.should_select {
                    continue;
                }

                let result_table = result_schema
                    .tables
                    .entry(table.name.clone())
                    .or_insert_with(|| Table {
                        name: table.name.clone(),
                        description: table.description.clone(),
                        columns: HashMap::new(),
                        should_delete: false,
                        should_select: false,
                    });

                for col in columns {
                    result_table.columns.insert(
                        col.name.clone(),
                        Column {
                            name: col.name.clone(),
                            data_type: col.data_type.clone(),
                            description: col.description.clone(),
                            sample_values: col.sample_values.clone(),
                            should_select: true,
                            should_delete: false,
                        },
                    );
                }
            }
        }
    }

    result
}

pub fn prune_data_catalog_batch(
    data_catalog_batch: &mut Database,
    deletion_set: DeletionSet,
    selection_set: SelectionSet,
) {
    for schema_tables in deletion_set.obviously_irrelevant_tables {
        if schema_tables.database == data_catalog_batch.name {
            for table_name in &schema_tables.tables {
                let _ = data_catalog_batch.delete_table(&schema_tables.schema, table_name);
            }
        }
    }
    for table_columns in deletion_set.obviously_irrelevant_columns {
        if table_columns.database == data_catalog_batch.name {
            for column_name in &table_columns.columns {
                let _ = data_catalog_batch.delete_column(
                    &table_columns.schema,
                    &table_columns.table,
                    column_name,
                );
            }
        }
    }

    for schema_tables in selection_set.relevant_tables {
        if schema_tables.database == data_catalog_batch.name {
            for table_name in &schema_tables.tables {
                let _ = data_catalog_batch.select_table(&schema_tables.schema, table_name);
            }
        }
    }
    for table_columns in selection_set.relevant_columns {
        if table_columns.database == data_catalog_batch.name {
            for column_name in &table_columns.columns {
                let _ = data_catalog_batch.select_column(
                    &table_columns.schema,
                    &table_columns.table,
                    column_name,
                );
            }
        }
    }
}

pub async fn get_data_catalog_batches(
    connection: &entity::connection::Model,
    batch_size: u32,
) -> Result<Vec<Database>, GetDataCatalogError> {
    let catalog = get_data_catalog_of_connection(connection).await?;
    let mut table_entries: Vec<(String, String, bool, bool, String, Table)> = Vec::new();
    for (_, schema) in catalog.schemas.iter() {
        for (table_name, table) in schema.tables.iter() {
            table_entries.push((
                schema.name.clone(),
                schema.description.clone(),
                schema.should_delete,
                schema.should_select,
                table_name.clone(),
                table.clone(),
            ));
        }
    }
    let batch_size_usize = (batch_size as usize).max(1);
    let mut batches = Vec::new();
    for chunk in table_entries.chunks(batch_size_usize) {
        let mut schema_map: HashMap<String, Schema> = HashMap::new();
        for (
            schema_name,
            schema_desc,
            schema_should_delete,
            schema_should_select,
            table_name,
            table,
        ) in chunk
        {
            let batch_schema = schema_map
                .entry(schema_name.clone())
                .or_insert_with(|| Schema {
                    name: schema_name.clone(),
                    description: schema_desc.clone(),
                    tables: HashMap::new(),
                    should_delete: *schema_should_delete,
                    should_select: *schema_should_select,
                });
            batch_schema
                .tables
                .insert(table_name.clone(), table.clone());
        }
        batches.push(Database {
            name: catalog.name.clone(),
            description: catalog.description.clone(),
            schemas: schema_map,
        });
    }
    if batches.is_empty() {
        batches.push(Database {
            name: catalog.name.clone(),
            description: catalog.description.clone(),
            schemas: HashMap::new(),
        });
    }
    Ok(batches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_pruned_batches_single_batch() {
        let batch = Database::new(
            "db1",
            "Single batch",
            vec![Schema::new(
                "public",
                "Default schema",
                vec![Table::new(
                    "users",
                    "User table",
                    vec![Column::entry("id", "Primary key", "integer", true, false)],
                    false,
                    false,
                )],
                false,
                false,
            )],
        );
        let result = join_pruned_batches(vec![batch]);
        assert_eq!(result.name, "db1");
        assert_eq!(result.description, "Single batch");
        assert_eq!(result.schemas.len(), 1);
        let schema = result.schemas.get("public").unwrap();
        assert_eq!(schema.tables.len(), 1);
        let table = schema.tables.get("users").unwrap();
        assert_eq!(table.columns.len(), 1);
        let col = table.columns.get("id").unwrap();
        assert!(col.should_select);
        assert!(!col.should_delete);
    }

    #[test]
    fn join_pruned_batches_multiple_batches_merges_schemas() {
        let batch1 = Database::new(
            "db1",
            "First batch",
            vec![Schema::new(
                "public",
                "Schema from batch 1",
                vec![Table::new(
                    "users",
                    "Users table",
                    vec![Column::entry("id", "ID col", "integer", true, false)],
                    false,
                    false,
                )],
                false,
                false,
            )],
        );
        let batch2 = Database::new(
            "db2",
            "Second batch",
            vec![Schema::new(
                "public",
                "Schema from batch 2",
                vec![Table::new(
                    "orders",
                    "Orders table",
                    vec![Column::entry("order_id", "Order ID", "text", true, false)],
                    false,
                    false,
                )],
                false,
                false,
            )],
        );
        let result = join_pruned_batches(vec![batch1, batch2]);
        assert_eq!(result.name, "db1");
        assert_eq!(result.description, "First batch");
        assert_eq!(result.schemas.len(), 1);
        let schema = result.schemas.get("public").unwrap();
        assert_eq!(schema.tables.len(), 2);
        assert!(schema.tables.contains_key("users"));
        assert!(schema.tables.contains_key("orders"));
    }

    #[test]
    fn join_pruned_batches_excludes_column_when_unselected_and_deleted() {
        let batch = Database::new(
            "db1",
            "Desc",
            vec![Schema::new(
                "public",
                "Schema",
                vec![Table::new(
                    "t1",
                    "Table",
                    vec![
                        Column::entry("keep1", "Keep selected", "text", true, false),
                        Column::entry(
                            "exclude",
                            "Exclude: unselected and deleted",
                            "text",
                            false,
                            true,
                        ),
                        Column::entry(
                            "keep2",
                            "Keep: unselected but not deleted",
                            "text",
                            false,
                            false,
                        ),
                    ],
                    false,
                    false,
                )],
                false,
                false,
            )],
        );
        let result = join_pruned_batches(vec![batch]);
        let table = result
            .schemas
            .get("public")
            .unwrap()
            .tables
            .get("t1")
            .unwrap();
        assert!(table.columns.contains_key("keep1"));
        assert!(!table.columns.contains_key("exclude"));
        assert!(table.columns.contains_key("keep2"));
    }

    #[test]
    fn join_pruned_batches_includes_table_when_should_select_even_without_columns() {
        let batch = Database::new(
            "db1",
            "Desc",
            vec![Schema::new(
                "public",
                "Schema",
                vec![Table::new(
                    "empty_table",
                    "Table with no included columns",
                    vec![Column::entry(
                        "excluded",
                        "Unselected and deleted",
                        "text",
                        false,
                        true,
                    )],
                    true,
                    false,
                )],
                false,
                false,
            )],
        );
        let result = join_pruned_batches(vec![batch]);
        assert!(result
            .schemas
            .get("public")
            .unwrap()
            .tables
            .contains_key("empty_table"));
    }

    #[test]
    fn join_pruned_batches_excludes_table_when_no_columns_and_not_selected() {
        let batch = Database::new(
            "db1",
            "Desc",
            vec![Schema::new(
                "public",
                "Schema",
                vec![
                    Table::new(
                        "users",
                        "Has columns",
                        vec![Column::entry("id", "ID", "integer", true, false)],
                        false,
                        false,
                    ),
                    Table::new(
                        "empty_table",
                        "Table with no included columns",
                        vec![Column::entry(
                            "excluded",
                            "Unselected and deleted",
                            "text",
                            false,
                            true,
                        )],
                        false,
                        false,
                    ),
                ],
                false,
                false,
            )],
        );
        let result = join_pruned_batches(vec![batch]);
        assert!(result
            .schemas
            .get("public")
            .unwrap()
            .tables
            .contains_key("users"));
        assert!(!result
            .schemas
            .get("public")
            .unwrap()
            .tables
            .contains_key("empty_table"));
    }

    #[test]
    fn join_pruned_batches_includes_schema_when_should_select() {
        let batch = Database::new(
            "db1",
            "Desc",
            vec![Schema::new(
                "analytics",
                "Analytics schema",
                vec![Table::new(
                    "empty_table",
                    "No columns",
                    vec![Column::entry("excluded", "Excluded", "text", false, true)],
                    false,
                    false,
                )],
                true,
                false,
            )],
        );
        let result = join_pruned_batches(vec![batch]);
        assert!(result.schemas.contains_key("analytics"));
    }

    #[test]
    fn join_pruned_batches_excludes_schema_when_not_selected_and_no_tables() {
        let batch = Database::new(
            "db1",
            "Desc",
            vec![Schema::new(
                "irrelevant",
                "Irrelevant schema",
                vec![Table::new(
                    "empty_table",
                    "No columns",
                    vec![Column::entry("excluded", "Excluded", "text", false, true)],
                    false,
                    false,
                )],
                false,
                false,
            )],
        );
        let result = join_pruned_batches(vec![batch]);
        assert!(!result.schemas.contains_key("irrelevant"));
    }

    #[test]
    fn join_pruned_batches_same_table_across_batches_last_columns_win() {
        let batch1 = Database::new(
            "db1",
            "First",
            vec![Schema::new(
                "public",
                "Schema",
                vec![Table::new(
                    "users",
                    "Table",
                    vec![Column::entry(
                        "id",
                        "ID from batch1",
                        "integer",
                        true,
                        false,
                    )],
                    false,
                    false,
                )],
                false,
                false,
            )],
        );
        let batch2 = Database::new(
            "db2",
            "Second",
            vec![Schema::new(
                "public",
                "Schema",
                vec![Table::new(
                    "users",
                    "Table",
                    vec![
                        Column::entry("id", "ID from batch2", "integer", true, false),
                        Column::entry("email", "Email", "text", true, false),
                    ],
                    false,
                    false,
                )],
                false,
                false,
            )],
        );
        let result = join_pruned_batches(vec![batch1, batch2]);
        let table = result
            .schemas
            .get("public")
            .unwrap()
            .tables
            .get("users")
            .unwrap();
        assert_eq!(table.columns.len(), 2);
        assert_eq!(
            table.columns.get("id").unwrap().description,
            "ID from batch2"
        );
        assert!(table.columns.contains_key("email"));
    }

    #[test]
    fn join_pruned_batches_multiple_schemas() {
        let batch = Database::new(
            "db1",
            "Desc",
            vec![
                Schema::new(
                    "public",
                    "Public schema",
                    vec![Table::new(
                        "users",
                        "Users",
                        vec![Column::entry("id", "ID", "integer", true, false)],
                        false,
                        false,
                    )],
                    false,
                    false,
                ),
                Schema::new(
                    "analytics",
                    "Analytics schema",
                    vec![Table::new(
                        "metrics",
                        "Metrics",
                        vec![Column::entry("value", "Value", "text", true, false)],
                        false,
                        false,
                    )],
                    false,
                    false,
                ),
            ],
        );
        let result = join_pruned_batches(vec![batch]);
        assert_eq!(result.schemas.len(), 2);
        assert!(result.schemas.contains_key("public"));
        assert!(result.schemas.contains_key("analytics"));
    }

    #[test]
    fn join_pruned_batches_empty_schemas_in_batch() {
        let batch = Database::new(
            "db1",
            "Desc",
            vec![Schema::new("public", "Empty schema", vec![], false, false)],
        );
        let result = join_pruned_batches(vec![batch]);
        assert_eq!(result.schemas.len(), 0);
    }

    #[test]
    fn join_pruned_batches_batch_with_no_schemas() {
        let batch = Database::new("db1", "Desc", vec![]);
        let result = join_pruned_batches(vec![batch]);
        assert_eq!(result.name, "db1");
        assert_eq!(result.description, "Desc");
        assert_eq!(result.schemas.len(), 0);
    }

    #[test]
    fn join_pruned_batches_result_columns_normalized_to_selected() {
        let batch = Database::new(
            "db1",
            "Desc",
            vec![Schema::new(
                "public",
                "Schema",
                vec![Table::new(
                    "t1",
                    "Table",
                    vec![Column::entry("id", "ID", "integer", false, false)],
                    false,
                    false,
                )],
                false,
                false,
            )],
        );
        let result = join_pruned_batches(vec![batch]);
        let col = result
            .schemas
            .get("public")
            .unwrap()
            .tables
            .get("t1")
            .unwrap()
            .columns
            .get("id")
            .unwrap();
        assert!(col.should_select);
        assert!(!col.should_delete);
    }

    #[test]
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    fn join_pruned_batches_empty_batches_panics() {
        join_pruned_batches(vec![]);
    }

    #[test]
    fn join_pruned_batches_column_deleted_but_selected_is_included() {
        let batch = Database::new(
            "db1",
            "Desc",
            vec![Schema::new(
                "public",
                "Schema",
                vec![Table::new(
                    "t1",
                    "Table",
                    vec![Column::entry(
                        "id",
                        "Selected but also deleted",
                        "integer",
                        true,
                        true,
                    )],
                    false,
                    false,
                )],
                false,
                false,
            )],
        );
        let result = join_pruned_batches(vec![batch]);
        assert!(result
            .schemas
            .get("public")
            .unwrap()
            .tables
            .get("t1")
            .unwrap()
            .columns
            .contains_key("id"));
    }
}
