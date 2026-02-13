pub fn build_pg_bulk_insert_query<'a>(
    table_name: &str,
    columns: &[&str],
    data: &[Vec<&'a (dyn tokio_postgres::types::ToSql + Sync + 'a)>], // Already trait objects
    timestamp_literal: &str,
) -> (
    String,
    Vec<&'a (dyn tokio_postgres::types::ToSql + Sync + 'a)>,
) {
    let mut query = format!(
        "INSERT INTO {} ({}, created_at, updated_at) VALUES ",
        table_name,
        columns.join(", ")
    );

    let mut all_params: Vec<&'a (dyn tokio_postgres::types::ToSql + Sync + 'a)> = Vec::new();
    let mut placeholders = Vec::new();

    let row_len = data[0].len(); // Assuming all rows have same length

    for (i, row) in data.iter().enumerate() {
        let mut row_placeholders: Vec<String> = Vec::new();
        // For regular columns
        for j in 0..row_len {
            row_placeholders.push(format!("${}", i * row_len + j + 1));
        }
        // For created_at (shared timestamp)
        row_placeholders.push(format!("'{}'::TIMESTAMP", timestamp_literal));
        // For updated_at (shared timestamp)
        row_placeholders.push(format!("'{}'::TIMESTAMP", timestamp_literal));

        placeholders.push(format!("({})", row_placeholders.join(", ")));
        all_params.extend(row.iter());
    }

    query.push_str(&placeholders.join(", "));
    query.push_str(
        " ON CONFLICT (zip_code, prefecture_id, city, town)
        DO UPDATE SET
        prefecture = EXCLUDED.prefecture,
        town = EXCLUDED.town,
        updated_at = EXCLUDED.updated_at;",
    );
    // println!("Generated Query: {}", query);
    (query, all_params)
}
