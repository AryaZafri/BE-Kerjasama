use actix_web::{HttpResponse, Responder, web};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{map_get, query_ch};

pub async fn tren_komoditas_utama(info: web::Json<Value>) -> impl Responder {
    let param = info.into_inner();
    
    let category = map_get("category", param.clone())
        .as_str()
        .unwrap_or("")
        .to_owned();

    let year = map_get("year", param.clone())
        .as_str()
        .unwrap_or("")
        .to_owned();

    let mitra_kerja_sama = map_get("mitra_kerja_sama", param.clone())
        .as_str()
        .unwrap_or("")
        .to_owned();

    let where_mitra_kerja_sama = if !mitra_kerja_sama.is_empty() {
        format!("AND mitra_kerja_sama = '{}'", mitra_kerja_sama.replace("'", "''"))
    } else {
        String::new()
    };

    let tables = if category.is_empty() {
        vec![("export", "Ekspor"), ("import", "Impor")]
    } else {
        let table = if category == "Ekspor" { "export" } else { "import" };
        let label = if category == "Ekspor" { "Ekspor" } else { "Impor" };
        vec![(table, label)]
    };

    let mut result_by_category = Vec::new();

    for (table, cat_label) in tables {
        let query = format!(
            "SELECT
                hs,
                year,
                produk,
                value
            FROM db_kerjasama.{}
            WHERE TRUE
                {}
            ORDER BY year ASC, value DESC
            FORMAT JSON;",
            table,
            where_mitra_kerja_sama
        );

        let res_query = query_ch(query).await.unwrap_or(Value::Null);
        let data = map_get("data", res_query)
            .as_array()
            .unwrap_or(&vec![])
            .to_owned();

        let mut year_map: HashMap<String, Vec<Value>> = HashMap::new();
        for item in data.iter() {
            let item_year = map_get("year", item.clone())
                .as_str()
                .unwrap_or("")
                .to_string();
            year_map.entry(item_year).or_insert_with(Vec::new).push(item.clone());
        }

        let mut years: Vec<_> = year_map.keys().cloned().collect();
        years.sort();

        let reference_year = if !year.is_empty() && year_map.contains_key(&year) {
            year.clone()
        } else if let Some(latest) = years.last() {
            latest.clone()
        } else {
            String::new()
        };

        let top_5_hs: Vec<(String, String)> = year_map
            .get(&reference_year)
            .map(|data| {
                data.iter()
                    .take(5)
                    .map(|item| {
                        let hs = map_get("hs", item.clone())
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        let produk = map_get("produk", item.clone())
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        (hs, produk)
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut hs_value_by_year: HashMap<String, HashMap<String, f64>> = HashMap::new();
        for (yr, items) in &year_map {
            let mut hs_map: HashMap<String, f64> = HashMap::new();
            for item in items {
                let hs = map_get("hs", item.clone())
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let value = map_get("value", item.clone())
                    .as_f64()
                    .unwrap_or(0.0);
                hs_map.insert(hs, value);
            }
            hs_value_by_year.insert(yr.clone(), hs_map);
        }

        let mut result_by_year = Vec::new();

        for (idx, yr) in years.iter().enumerate() {
            let prev_year = if idx > 0 { Some(&years[idx - 1]) } else { None };
            let prev_hs_map = prev_year.and_then(|py| hs_value_by_year.get(py));
            let current_hs_map = hs_value_by_year.get(yr);

            let year_data = build_year_data(
                yr.clone(),
                &top_5_hs,
                current_hs_map,
                prev_hs_map
            );

            result_by_year.push(year_data);
        }

        result_by_category.push(json!({
            "category": cat_label,
            "years": result_by_year
        }));
    }

    let result = json!({
        "data": result_by_category
    });

    HttpResponse::Ok().json(result)
}

fn format_value(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("{:.1}B", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.1}M", value / 1_000.0)
    } else {
        format!("{:.1}K", value)
    }
}

fn build_year_data(
    year: String,
    top_5_hs: &[(String, String)],
    current_hs_map: Option<&HashMap<String, f64>>,
    prev_hs_map: Option<&HashMap<String, f64>>
) -> Value {
    let result_data: Vec<Value> = top_5_hs.iter()
        .map(|(hs, produk)| {
            let value = current_hs_map
                .and_then(|m| m.get(hs))
                .copied()
                .unwrap_or(0.0);
            
            let values = format_value(value);

            let (percentage, status) = match prev_hs_map {
                Some(prev_map) => {
                    match prev_map.get(hs) {
                        Some(&prev_value) if prev_value > 0.0 => {
                            let pct_change = ((value - prev_value) / prev_value) * 100.0;
                            let status = if pct_change > 0.0 {
                                "Increase"
                            } else if pct_change < 0.0 {
                                "Decrease"
                            } else {
                                "Stable"
                            };
                            (format!("{:.2}%", pct_change), status.to_string())
                        },
                        Some(_) => ("-".to_string(), "N/A".to_string()),
                        None => {
                            if value > 0.0 {
                                ("-".to_string(), "New".to_string())
                            } else {
                                ("-".to_string(), "N/A".to_string())
                            }
                        },
                    }
                },
                None => ("-".to_string(), "N/A".to_string()),
            };

            json!({
                "hs": hs,
                "produk": produk,
                "value": values,
                "percentage": percentage,
                "status": status
            })
        })
        .collect();

    json!({
        "year": year,
        "data": result_data
    })
}