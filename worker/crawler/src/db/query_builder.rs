pub fn build_pg_bulk_insert_query<'a>(
    table_name: &str,
    columns: &[&str],
    data: &[Vec<&'a (dyn tokio_postgres::types::ToSql + Sync + 'a)>], // Already trait objects
) -> (
    String,
    Vec<&'a (dyn tokio_postgres::types::ToSql + Sync + 'a)>,
) {
    let mut query = format!(
        "INSERT INTO {} ({}) VALUES ",
        table_name,
        columns.join(", ")
    );

    let mut all_params: Vec<&'a (dyn tokio_postgres::types::ToSql + Sync + 'a)> = Vec::new();
    let mut placeholders = Vec::new();

    for (i, row) in data.iter().enumerate() {
        let mut row_placeholders: Vec<String> = Vec::new();
        for (j, _) in row.iter().enumerate() {
            row_placeholders.push(format!("${}", i * row.len() + j + 1));
        }
        placeholders.push(format!("({})", row_placeholders.join(", ")));
        all_params.extend(row.iter());
    }

    query.push_str(&placeholders.join(", "));
    query.push_str(
        " ON CONFLICT (zip_code, prefecture_id, city, town)
        DO UPDATE SET
        prefecture = EXCLUDED.prefecture,
        town = EXCLUDED.town,
        updated_at = CURRENT_TIMESTAMP;",
    );
    (query, all_params)
}
