#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use std::time::Duration;

use comfy_table::{Attribute, Cell, ContentArrangement, Table, presets};

use crate::metrics::BenchmarkResult;

/// Creates a dynamic table with bold headers (same style as miden-cli)
fn create_dynamic_table(headers: &[&str]) -> Table {
    let header_cells = headers
        .iter()
        .map(|header| Cell::new(header).add_attribute(Attribute::Bold))
        .collect::<Vec<_>>();

    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(header_cells);

    table
}

/// Prints benchmark results as a pretty table
pub fn print_results(results: &[BenchmarkResult], title: &str, total_duration: Duration) {
    println!();

    let mut table = create_dynamic_table(&[title, "Mean", "Min", "Max"]);

    for result in results {
        let mut row = vec![
            result.name.clone(),
            format_duration(result.mean()),
            format_duration(result.min()),
            format_duration(result.max()),
        ];

        // Add output size info to the benchmark name if present
        if let Some(size) = result.output_size {
            row[0] = format!("{}\n  Output: {}", result.name, format_size(size));
        }

        table.add_row(row);
    }

    println!("{table}");

    // Summary line
    println!(
        "\nTotal benchmarks: {} | Total time: {}",
        results.len(),
        format_duration(total_duration)
    );
}

fn format_duration(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms < 1.0 {
        format!("{:.2}us", d.as_nanos() as f64 / 1000.0)
    } else if ms < 1000.0 {
        format!("{ms:.2}ms")
    } else {
        format!("{:.2}s", d.as_secs_f64())
    }
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
