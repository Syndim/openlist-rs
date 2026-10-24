use anyhow::Result;
use openlist::models::ApiResponse;
use serde::Serialize;

pub fn emit_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

/// Build a two-column table row, rendering `None` as an empty value.
pub fn field<T: std::fmt::Display>(name: &str, value: &Option<T>) -> Vec<String> {
    vec![
        name.to_string(),
        value.as_ref().map(ToString::to_string).unwrap_or_default(),
    ]
}

/// Print a plain API response carrying no meaningful payload.
pub fn emit_message(resp: &ApiResponse<serde_json::Value>) -> Result<()> {
    match &resp.data {
        Some(data) if !data.is_null() => println!("{}: {}", resp.message, data),
        _ => println!("{}", resp.message),
    }
    Ok(())
}

/// Print a left-aligned table with a header separator.
pub fn table(headers: &[&str], rows: Vec<Vec<String>>) {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.chars().count());
            }
        }
    }

    print_row(headers.iter().map(|h| h.to_string()).collect(), &widths);
    let separator: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
    print_row(separator, &widths);

    for row in rows {
        print_row(row, &widths);
    }
}

fn print_row(cells: Vec<String>, widths: &[usize]) {
    let line = cells
        .iter()
        .enumerate()
        .map(|(i, cell)| {
            format!(
                "{:<width$}",
                cell,
                width = widths.get(i).copied().unwrap_or(0)
            )
        })
        .collect::<Vec<_>>()
        .join("  ");
    println!("{}", line.trim_end());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_missing_values_as_empty() {
        assert_eq!(field("id", &None::<i64>), vec!["id", ""]);
        assert_eq!(field("id", &Some(7)), vec!["id", "7"]);
    }
}
