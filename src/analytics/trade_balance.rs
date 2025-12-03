use actix_web::{HttpResponse, Responder, web};
use serde_json::{json, Value};

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

pub async fn trade_balance(info: web::Json<Value>) -> impl Responder {
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
    
    for yr in &years {
        let query_part = format!(
            "SELECT
                'Ekspor' AS category,
                year,
                SUM(value) AS value
            FROM db_kerjasama.export
            WHERE TRUE
                AND year = '{}'
                {}
            GROUP BY year",
            yr,
            where_mitra_kerja_sama
        );
        union_queries.push(query_part);
    }

    for yr in &years {
        let query_part = format!(
            "SELECT
                'Impor' AS category,
                year,
                SUM(value) AS value
            FROM db_kerjasama.import
            WHERE TRUE
                AND year = '{}'
                {}
            GROUP BY year",
            yr,
            where_mitra_kerja_sama
        );
        union_queries.push(query_part);
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

    let mut year_data: std::collections::HashMap<String, (f64, f64)> = std::collections::HashMap::new();
    
    for item in data {
        let yr = map_get("year", item.clone())
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let category = map_get("category", item.clone())
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let value = map_get("value", item.clone())
            .as_f64()
            .unwrap_or(0.0);
        
        let entry = year_data.entry(yr).or_insert((0.0, 0.0));
        
        if category == "Ekspor" {
            entry.0 = value;
        } else if category == "Impor" {
            entry.1 = value;
        }
    }

    let mut result_data = Vec::new();
    
    for yr in &years {
        if let Some((export, import)) = year_data.get(yr) {
            let balance = export - import;
            let total_trade = export + import;
            
            let percentage = if total_trade > 0.0 {
                (balance / total_trade * 100.0).round() as i32
            } else {
                0
            };
            
            let status = if balance > 0.0 {
                "Surplus"
            } else if balance < 0.0 {
                "Deficit"
            } else {
                "Balanced"
            };
            
            result_data.push(json!({
                "year": yr,
                "ekspor": format_value(*export),
                "impor": format_value(*import),
                "percentage": percentage,
                "status": status
            }));
        }
    }

    HttpResponse::Ok().json(result_data)
}