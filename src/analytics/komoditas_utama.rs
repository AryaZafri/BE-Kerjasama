use actix_web::{HttpResponse, Responder, web};
use serde_json::{json, Value};

use crate::{map_get, query_ch};

pub async fn komoditas_utama(info: web::Json<Value>) -> impl Responder {
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

    let where_mitra_kerja_sama = if mitra_kerja_sama != "" {
        format!("AND mitra_kerja_sama = '{}'", mitra_kerja_sama.replace("'", "''"))
    } else {
        String::new()
    };

    let where_year = if year != "" {
        format!("AND year = '{}'", year.replace("'", "''"))
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
                {}
            ORDER BY year ASC, value DESC
            FORMAT JSON;",
            table,
            where_year,
            where_mitra_kerja_sama
        );

        let res_query = query_ch(query).await.unwrap_or(Value::Null);
        let data = map_get("data", res_query)
            .as_array()
            .unwrap_or(&vec![])
            .to_owned();

        let mut year_map: std::collections::HashMap<String, Vec<Value>> = std::collections::HashMap::new();
        
        for item in data.iter() {
            let item_year = map_get("year", item.clone())
                .as_str()
                .unwrap_or("")
                .to_string();
            
            year_map.entry(item_year).or_insert_with(Vec::new).push(item.clone());
        }

        let mut years: Vec<_> = year_map.keys().collect();
        years.sort();

        let mut result_by_year = Vec::new();

        for y in years {
            let year_data = &year_map[y];
            result_by_year.push(process_year_data(y.to_string(), year_data.clone()));
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

fn process_year_data(year: String, year_data: Vec<Value>) -> Value {
    let total_value: f64 = year_data.iter()
        .filter_map(|item| map_get("value", item.clone()).as_f64())
        .sum();

    let top_5: Vec<Value> = year_data.iter().take(5).cloned().collect();
    let top_5_sum: f64 = top_5.iter()
        .filter_map(|item| map_get("value", item.clone()).as_f64())
        .sum();
    
    let others_value = total_value - top_5_sum;

    let mut result_data: Vec<Value> = top_5.iter()
        .map(|item| {
            let hs = map_get("hs", item.clone())
                .as_str()
                .unwrap_or("")
                .to_string();
            
            let produk = map_get("produk", item.clone())
                .as_str()
                .unwrap_or("")
                .to_string();
            
            let value = map_get("value", item.clone())
                .as_f64()
                .unwrap_or(0.0);
            
            let value_int = (value * 1000.0) as i64;
            
            let percentage = if total_value > 0.0 {
                (value / total_value) * 100.0
            } else {
                0.0
            };

            json!({
                "hs": hs,
                "produk": produk,
                "value": value_int,
                "percentage": format!("{:.0}%", percentage)
            })
        })
        .collect();

    if year_data.len() > 5 {
        let others_percentage = if total_value > 0.0 {
            (others_value / total_value) * 100.0
        } else {
            0.0
        };

        let others_value_int = others_value as i64;

        result_data.push(json!({
            "hs": "-",
            "produk": "Others",
            "value": others_value_int,
            "percentage": format!("{:.1}%", others_percentage)
        }));
    }

    json!({
        "year": year,
        "data": result_data
    })
}