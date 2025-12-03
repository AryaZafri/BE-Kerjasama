use actix_web::{HttpResponse, Responder, web};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{map_get, query_ch};

fn format_value(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("{:.1}B", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.1}M", value / 1_000.0)
    } else {
        format!("{:.1}K", value)
    }
}

pub async fn tren_ekspor_impor(info: web::Json<Value>) -> impl Responder {
    let param = info.into_inner();
    
    let mitra_kerja_sama = map_get("mitra_kerja_sama", param.clone())
        .as_str()
        .unwrap_or("")
        .to_owned();

    let where_mitra_kerja_sama = if mitra_kerja_sama != "" {
        format!("AND mitra_kerja_sama = '{}'", mitra_kerja_sama.replace("'", "''"))
    } else {
        String::new()
    };

    let year = map_get("year", param.clone())
        .as_str()
        .unwrap_or("")
        .to_owned();

    let category = map_get("category", param.clone())
        .as_str()
        .unwrap_or("")
        .to_owned();
    
    let tables: Vec<(&str, &str)> = match category.as_str() {
        "Ekspor" => vec![("export", "Ekspor")],
        "Impor" => vec![("import", "Impor")],
        _ => vec![("export", "Ekspor"), ("import", "Impor")],
    };

    let years_query = "
        SELECT DISTINCT year FROM (
            SELECT year FROM db_kerjasama.export
            UNION ALL
            SELECT year FROM db_kerjasama.import
        )
        ORDER BY year
        FORMAT JSON;
    ";
    
    let years_result = query_ch(years_query.to_string()).await.unwrap_or(Value::Null);
    let years_data = map_get("data", years_result)
        .as_array()
        .unwrap_or(&vec![])
        .to_owned();
    
    let years: Vec<String> = if year != "" {
        vec![year.clone()]
    } else {
        years_data.iter()
            .filter_map(|v| map_get("year", v.clone()).as_str().map(|s| s.to_string()))
            .collect()
    };

    let mut union_queries = Vec::new();
    
    for (table, cat_label) in &tables {
        for yr in &years {
            let query_part = format!(
                "SELECT
                    '{}' AS category,
                    year,
                    SUM(value) AS value
                FROM db_kerjasama.{}
                WHERE TRUE
                    AND year = '{}'
                    {}
                GROUP BY year",
                cat_label,
                table,
                yr,
                where_mitra_kerja_sama
            );
            union_queries.push(query_part);
        }
    }

    let query = format!(
        "{}
        ORDER BY year, category
        FORMAT JSON;",
        union_queries.join("\nUNION ALL\n")
    );

    let res_query = query_ch(query).await.unwrap_or(Value::Null);
    let data = map_get("data", res_query)
        .as_array()
        .unwrap_or(&vec![])
        .to_owned();

    let mut yearly_data: HashMap<String, HashMap<String, f64>> = HashMap::new();
    
    for item in data {
        let cat = map_get("category", item.clone())
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let year_val = map_get("year", item.clone())
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let value_raw = map_get("value", item.clone())
            .as_f64()
            .unwrap_or(0.0);
        
        yearly_data.entry(year_val)
            .or_insert_with(HashMap::new)
            .insert(cat, value_raw);
    }

    let mut result_data: Vec<Value> = Vec::new();
    let mut total_ekspor = 0.0;
    let mut total_impor = 0.0;

    for (idx, year_val) in years.iter().enumerate() {
        if let Some(categories) = yearly_data.get(year_val) {
            let ekspor = categories.get("Ekspor").unwrap_or(&0.0);
            let impor = categories.get("Impor").unwrap_or(&0.0);
            
            total_ekspor += ekspor;
            total_impor += impor;
            
            let (ekspor_change, impor_change) = if idx > 0 {
                let prev_year = &years[idx - 1];
                if let Some(prev_categories) = yearly_data.get(prev_year) {
                    let prev_ekspor = *prev_categories.get("Ekspor").unwrap_or(&0.0);
                    let prev_impor = *prev_categories.get("Impor").unwrap_or(&0.0);
                    
                    let ekspor_pct = if prev_ekspor > 0.0 {
                        ((ekspor - prev_ekspor) / prev_ekspor) * 100.0
                    } else if ekspor > &0.0 {
                        100.0
                    } else {
                        0.0
                    };
                    
                    let impor_pct = if prev_impor > 0.0 {
                        ((impor - prev_impor) / prev_impor) * 100.0
                    } else if impor > &0.0 {
                        100.0
                    } else {
                        0.0
                    };
                    
                    (Some(ekspor_pct), Some(impor_pct))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };
            
            let ekspor_percentage_str = match ekspor_change {
                Some(pct) => {
                    if pct > 0.0 {
                        format!("+{:.1}%", pct)
                    } else {
                        format!("{:.1}%", pct)
                    }
                },
                None => "-".to_string()
            };
            
            let impor_percentage_str = match impor_change {
                Some(pct) => {
                    if pct > 0.0 {
                        format!("+{:.1}%", pct)
                    } else {
                        format!("{:.1}%", pct)
                    }
                },
                None => "-".to_string()
            };
            
            let mut year_data = json!({
                "year": year_val
            });
            
            if category.is_empty() || category == "Ekspor" {
                year_data["ekspor"] = json!({
                    "value": format_value(*ekspor),
                    "percentage": ekspor_percentage_str
                });
            }
            
            if category.is_empty() || category == "Impor" {
                year_data["impor"] = json!({
                    "value": format_value(*impor),
                    "percentage": impor_percentage_str
                });
            }
            
            result_data.push(year_data);
        }
    }

    let result = json!({
        "data": result_data,
        "total": format_value(total_ekspor + total_impor)
    });

    HttpResponse::Ok().json(result)
}