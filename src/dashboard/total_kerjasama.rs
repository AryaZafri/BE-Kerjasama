use actix_web::{HttpResponse, Responder, web};
use serde_json::{json, Value};

use crate::{map_get, query_ch, to_i64};

pub async fn total_kerjasama(info: web::Json<Value>) -> impl Responder {
    let param = info.into_inner();
    
    let category = map_get("category", param.clone())
        .as_str()
        .unwrap_or("")
        .to_owned();

    let where_category = if category != "" {
        format!("AND category = '{}'", category.replace("'", "''"))
    } else {
        format!("")
    };

    let information = map_get("information", param.clone())
        .as_str()
        .unwrap_or("")
        .to_owned();

    let where_information = if information != "" {
        format!("AND multiIf(`year` != '', 'Berlaku', 'Proses') = '{}'", information.replace("'", "''"))
    } else {
        format!("")
    };

    let perundingan = map_get("perundingan", param.clone())
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_owned())
                .collect::<Vec<String>>()
        })
        .unwrap_or_else(Vec::new);

    let where_perundingan = if !perundingan.is_empty() {
        let escaped_values: Vec<String> = perundingan
            .iter()
            .map(|v| format!("'{}'", v.replace("'", "''")))
            .collect();
        format!("AND perundingan IN ({})", escaped_values.join(", "))
    } else {
        format!("--")
    };
    
    let query = format!(
        "SELECT
            category,
            COUNT(*) AS jumlah
        FROM db_kerjasama.kerjasama
        WHERE TRUE
        {where_category}
        {where_information}
        {where_perundingan}
        GROUP BY category
        ORDER BY category ASC
        FORMAT JSON;"
    );

    let res_query = query_ch(query).await.unwrap_or(Value::Null);
    let data = map_get("data", res_query)
        .as_array()
        .unwrap_or(&vec![])
        .to_owned();

    let categories = vec!["Regional", "Bilateral", "Multilateral"];
    
    let mut category_map: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    
    for cat in categories.iter() {
        category_map.insert(cat.to_string(), 0);
    }
    
    for item in data.iter() {
        let cat = map_get("category", item.clone())
            .as_str()
            .unwrap_or("")
            .to_owned();
        let jumlah = to_i64(map_get("jumlah", item.clone())) as u64;
        
        if categories.contains(&cat.as_str()) {
            category_map.insert(cat, jumlah);
        }
    }

    let total: u64 = category_map.values().sum();

    let result: Vec<Value> = categories.into_iter()
        .map(|category| {
            let jumlah = *category_map.get(category).unwrap_or(&0);
            
            let percentage = if total > 0 {
                ((jumlah as f64 / total as f64) * 100.0 * 100.0).round() / 100.0
            } else {
                0.0
            };

            json!({
                "category": category,
                "jumlah": jumlah,
                "percentage": percentage
            })
        })
        .collect();

    HttpResponse::Ok().json(result)
}