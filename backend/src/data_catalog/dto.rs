use std::collections::HashMap;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Database {
    pub name: String,
    pub description: String,
    pub schemas: HashMap<String, Schema>,
}

impl Database {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        schemas: impl IntoIterator<Item = Schema>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            schemas: schemas.into_iter().map(|s| (s.name.clone(), s)).collect(),
        }
    }

    pub fn to_display_string(&self) -> Result<String, serde_json::Error> {
        let mut schemas: Vec<SchemaDisplay> =
            self.schemas.values().map(SchemaDisplay::from).collect();
        schemas.sort_by_key(|s| s.name.clone());
        let display = DatabaseDisplay {
            name: self.name.clone(),
            description: self.description.clone(),
            schemas,
        };
        serde_json::to_string_pretty(&display)
    }

    pub fn get_schema(&mut self, schema_name: &str) -> Result<&mut Schema, String> {
        self.schemas
            .get_mut(schema_name)
            .ok_or_else(|| format!("Schema '{}' not found", schema_name))
    }

    pub fn delete_schema(&mut self, schema_name: &str) -> Result<(), String> {
        let schema = self.get_schema(schema_name)?;
        schema.delete_schema();
        Ok(())
    }

    pub fn delete_table(&mut self, schema_name: &str, table_name: &str) -> Result<(), String> {
        let schema = self.get_schema(schema_name)?;
        let table = schema.get_table(table_name)?;
        table.delete_table();
        Ok(())
    }

    pub fn delete_column(
        &mut self,
        schema_name: &str,
        table_name: &str,
        column_name: &str,
    ) -> Result<(), String> {
        let schema = self.get_schema(schema_name)?;
        let table = schema.get_table(table_name)?;
        let column = table.get_column(column_name)?;
        column.delete_column();
        Ok(())
    }

    pub fn select_schema(&mut self, schema_name: &str) -> Result<(), String> {
        let schema = self.get_schema(schema_name)?;
        schema.select_schema();
        Ok(())
    }

    pub fn select_table(&mut self, schema_name: &str, table_name: &str) -> Result<(), String> {
        let schema = self.get_schema(schema_name)?;
        let table = schema.get_table(table_name)?;
        table.select_table();
        Ok(())
    }

    pub fn select_column(
        &mut self,
        schema_name: &str,
        table_name: &str,
        column_name: &str,
    ) -> Result<(), String> {
        let schema = self.get_schema(schema_name)?;
        let table = schema.get_table(table_name)?;
        let column = table.get_column(column_name)?;
        column.select_column();
        Ok(())
    }

    pub fn get_table(&self, schema_name: &str, table_name: &str) -> Option<&Table> {
        self.schemas
            .get(schema_name)
            .and_then(|s| s.tables.get(table_name))
    }

    pub fn tables(&self) -> impl Iterator<Item = (&str, &str, &Table)> {
        self.schemas.iter().flat_map(|(schema_name, schema)| {
            schema
                .tables
                .iter()
                .map(move |(table_name, table)| (schema_name.as_str(), table_name.as_str(), table))
        })
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Schema {
    pub name: String,
    pub description: String,
    pub tables: HashMap<String, Table>,
    pub should_delete: bool,
    pub should_select: bool,
}

impl Schema {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        tables: impl IntoIterator<Item = Table>,
        should_select: bool,
        should_delete: bool,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            tables: tables.into_iter().map(|t| (t.name.clone(), t)).collect(),
            should_select,
            should_delete,
        }
    }

    pub fn get_table(&mut self, table_name: &str) -> Result<&mut Table, String> {
        self.tables
            .get_mut(table_name)
            .ok_or_else(|| format!("Table '{}' not found", table_name))
    }

    pub fn delete_schema(&mut self) {
        self.should_delete = true
    }

    pub fn select_schema(&mut self) {
        self.should_select = true
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Table {
    pub name: String,
    pub description: String,
    pub columns: HashMap<String, Column>,
    pub should_delete: bool,
    pub should_select: bool,
}

impl Table {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        columns: impl IntoIterator<Item = (impl Into<String>, Column)>,
        should_select: bool,
        should_delete: bool,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            columns: columns.into_iter().map(|(k, v)| (k.into(), v)).collect(),
            should_select,
            should_delete,
        }
    }

    pub fn get_column(&mut self, column_name: &str) -> Result<&mut Column, String> {
        self.columns
            .get_mut(column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))
    }

    pub fn delete_table(&mut self) {
        self.should_delete = true
    }

    pub fn select_table(&mut self) {
        self.should_select = true
    }

    pub fn columns_as_vec(&self) -> Vec<Column> {
        let mut columns: Vec<Column> = self.columns.values().cloned().collect();
        columns.sort_by_key(|c| c.name.clone());
        columns
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub description: String,
    pub sample_values: Vec<String>,
    pub should_delete: bool,
    pub should_select: bool,
}

impl Column {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        data_type: impl Into<String>,
        should_select: bool,
        should_delete: bool,
    ) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            data_type: data_type.into(),
            description: description.into(),
            sample_values: vec![],
            should_select,
            should_delete,
        }
    }

    pub fn with_samples(
        name: impl Into<String>,
        description: impl Into<String>,
        sample_values: impl IntoIterator<Item = impl Into<String>>,
        data_type: impl Into<String>,
        should_select: bool,
        should_delete: bool,
    ) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            data_type: data_type.into(),
            description: description.into(),
            sample_values: sample_values.into_iter().map(Into::into).collect(),
            should_select,
            should_delete,
        }
    }

    pub fn entry(
        name: impl Into<String>,
        description: impl Into<String>,
        data_type: impl Into<String>,
        should_select: bool,
        should_delete: bool,
    ) -> (String, Self) {
        let col = Self::new(name, description, data_type, should_select, should_delete);
        (col.name.clone(), col)
    }

    pub fn delete_column(&mut self) {
        self.should_delete = true
    }

    pub fn select_column(&mut self) {
        self.should_select = true
    }
}

#[derive(serde::Serialize)]
struct SchemaDisplay {
    name: String,
    description: String,
    tables: Vec<TableDisplay>,
}

#[derive(serde::Serialize)]
struct TableDisplay {
    name: String,
    description: String,
    columns: Vec<ColumnDisplay>,
}

#[derive(serde::Serialize)]
struct ColumnDisplay {
    name: String,
    description: String,
    sample_values: Vec<String>,
}

#[derive(serde::Serialize)]
struct DatabaseDisplay {
    name: String,
    description: String,
    schemas: Vec<SchemaDisplay>,
}

impl From<&Schema> for SchemaDisplay {
    fn from(s: &Schema) -> Self {
        let mut tables: Vec<TableDisplay> = s.tables.values().map(TableDisplay::from).collect();
        tables.sort_by_key(|t| t.name.clone());
        SchemaDisplay {
            name: s.name.clone(),
            description: s.description.clone(),
            tables,
        }
    }
}

impl From<&Table> for TableDisplay {
    fn from(t: &Table) -> Self {
        let mut columns: Vec<ColumnDisplay> = t.columns.values().map(ColumnDisplay::from).collect();
        columns.sort_by_key(|c| c.name.clone());
        TableDisplay {
            name: t.name.clone(),
            description: t.description.clone(),
            columns,
        }
    }
}

impl From<&Column> for ColumnDisplay {
    fn from(c: &Column) -> Self {
        ColumnDisplay {
            name: c.name.clone(),
            description: c.description.clone(),
            sample_values: c.sample_values.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn to_display_string_empty_database() {
        let db = Database::new("db1", "asdasd", std::iter::empty::<Schema>());
        let s = db.to_display_string().unwrap();
        let expected = indoc! { r#"
            {
              "name": "db1",
              "description": "asdasd",
              "schemas": []
            }
        "# }
        .trim_end();
        assert_eq!(s, expected);
    }

    #[test]
    fn to_display_string_schema_without_tables() {
        let db = Database::new(
            "db1",
            "asdasd",
            [Schema::new(
                "public",
                "public schema",
                std::iter::empty::<Table>(),
                false,
                false,
            )],
        );
        let s = db.to_display_string().unwrap();
        let expected = indoc! { r#"
            {
              "name": "db1",
              "description": "asdasd",
              "schemas": [
                {
                  "name": "public",
                  "description": "public schema",
                  "tables": []
                }
              ]
            }
        "# }
        .trim_end();
        assert_eq!(s, expected);
    }

    #[test]
    fn to_display_string_hashmap_keys_not_in_output() {
        let columns = [Column::with_samples(
            "id",
            "primary key",
            vec!["1", "2"],
            "integer",
            false,
            false,
        )];
        let tables = [Table::new(
            "users",
            "user table",
            columns.into_iter().map(|c| (c.name.clone(), c)),
            false,
            false,
        )];
        let db = Database::new(
            "db1",
            "asdasd",
            [Schema::new("public", "public schema", tables, false, false)],
        );
        let s = db.to_display_string().unwrap();
        let expected = indoc! { r#"
            {
              "name": "db1",
              "description": "asdasd",
              "schemas": [
                {
                  "name": "public",
                  "description": "public schema",
                  "tables": [
                    {
                      "name": "users",
                      "description": "user table",
                      "columns": [
                        {
                          "name": "id",
                          "description": "primary key",
                          "sample_values": [
                            "1",
                            "2"
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            }
        "# }
        .trim_end();
        assert_eq!(s, expected);
    }

    #[test]
    fn to_display_string_full_hierarchy() {
        let columns = [
            Column::with_samples(
                "id",
                "primary key",
                vec!["1", "2", "3"],
                "integer",
                false,
                false,
            ),
            Column::with_samples("email", "user email", vec!["a@b.com"], "text", false, false),
        ];
        let tables = [Table::new(
            "users",
            "user table",
            columns.into_iter().map(|c| (c.name.clone(), c)),
            false,
            false,
        )];
        let db = Database::new(
            "mydb",
            "test database",
            [Schema::new("public", "public schema", tables, false, false)],
        );
        let s = db.to_display_string().unwrap();
        let expected = indoc! { r#"
            {
              "name": "mydb",
              "description": "test database",
              "schemas": [
                {
                  "name": "public",
                  "description": "public schema",
                  "tables": [
                    {
                      "name": "users",
                      "description": "user table",
                      "columns": [
                        {
                          "name": "email",
                          "description": "user email",
                          "sample_values": [
                            "a@b.com"
                          ]
                        },
                        {
                          "name": "id",
                          "description": "primary key",
                          "sample_values": [
                            "1",
                            "2",
                            "3"
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            }
        "# }
        .trim_end();
        assert_eq!(s, expected);
    }

    #[test]
    fn to_display_string_multiple_schemas() {
        let db = Database::new(
            "db1",
            "desc",
            [
                Schema::new(
                    "schema_a",
                    "first",
                    std::iter::empty::<Table>(),
                    false,
                    false,
                ),
                Schema::new(
                    "schema_b",
                    "second",
                    std::iter::empty::<Table>(),
                    false,
                    false,
                ),
            ],
        );
        let s = db.to_display_string().unwrap();
        let expected = indoc! { r#"
            {
              "name": "db1",
              "description": "desc",
              "schemas": [
                {
                  "name": "schema_a",
                  "description": "first",
                  "tables": []
                },
                {
                  "name": "schema_b",
                  "description": "second",
                  "tables": []
                }
              ]
            }
        "# }
        .trim_end();
        assert_eq!(s, expected);
    }

    #[test]
    fn to_display_string_valid_json() {
        let columns = [Column::new("col1", "desc", "text", false, false)];
        let tables = [Table::new(
            "tbl1",
            "tbl desc",
            columns.into_iter().map(|c| (c.name.clone(), c)),
            false,
            false,
        )];
        let db = Database::new(
            "db1",
            "asdasd",
            [Schema::new("sch1", "schema desc", tables, false, false)],
        );
        let s = db.to_display_string().unwrap();
        let expected = indoc! { r#"
            {
              "name": "db1",
              "description": "asdasd",
              "schemas": [
                {
                  "name": "sch1",
                  "description": "schema desc",
                  "tables": [
                    {
                      "name": "tbl1",
                      "description": "tbl desc",
                      "columns": [
                        {
                          "name": "col1",
                          "description": "desc",
                          "sample_values": []
                        }
                      ]
                    }
                  ]
                }
              ]
            }
        "# }
        .trim_end();
        assert_eq!(s, expected);
    }
}
