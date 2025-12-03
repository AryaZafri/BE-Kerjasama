use actix_web::{HttpResponse, Responder, web};
use serde_json::{json, Value};

use crate::{map_get, query_ch};

pub async fn utilitas_kerjasama(info: web::Json<Value>) -> impl Responder {
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

    let query = format!("
        SELECT
            perundingan,
            tw1,
            tw2,
            tw3,
            tw4
        FROM db_kerjasama.kerjasama a
        LEFT JOIN (
            SELECT
                perundingan,
                tw1,
                tw2,
                tw3,
                tw4
            FROM db_kerjasama.utilization
        ) b ON a.perundingan = b.perundingan
        WHERE TRUE
        {where_category}
        {where_information}
        {where_perundingan}
        FORMAT JSON;"
    );

    let res_query = query_ch(query).await.unwrap_or(Value::Null);
    let raw_data = map_get("data", res_query)
        .as_array()
        .unwrap_or(&vec![])
        .to_vec();

    let mut tw1_data: Vec<Value> = Vec::new();
    let mut tw2_data: Vec<Value> = Vec::new();
    let mut tw3_data: Vec<Value> = Vec::new();
    let mut tw4_data: Vec<Value> = Vec::new();

    for item in raw_data {
        let perundingan = map_get("perundingan", item.clone())
            .as_str()
            .unwrap_or("")
            .to_string();

        if let Some(tw1) = map_get("tw1", item.clone()).as_f64() {
            tw1_data.push(json!({
                "perundingan": perundingan.clone(),
                "value": tw1
            }));
        }

        if let Some(tw2) = map_get("tw2", item.clone()).as_f64() {
            tw2_data.push(json!({
                "perundingan": perundingan.clone(),
                "value": tw2
            }));
        }

        if let Some(tw3) = map_get("tw3", item.clone()).as_f64() {
            tw3_data.push(json!({
                "perundingan": perundingan.clone(),
                "value": tw3
            }));
        }

        if let Some(tw4) = map_get("tw4", item.clone()).as_f64() {
            tw4_data.push(json!({
                "perundingan": perundingan.clone(),
                "value": tw4
            }));
        }
    }

    tw1_data.sort_by(|a, b| {
        let val_a = a["value"].as_f64().unwrap_or(0.0);
        let val_b = b["value"].as_f64().unwrap_or(0.0);
        let name_a = a["perundingan"].as_str().unwrap_or("");
        let name_b = b["perundingan"].as_str().unwrap_or("");
        
        val_b.partial_cmp(&val_a)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| name_a.cmp(name_b))
    });

    tw2_data.sort_by(|a, b| {
        let val_a = a["value"].as_f64().unwrap_or(0.0);
        let val_b = b["value"].as_f64().unwrap_or(0.0);
        let name_a = a["perundingan"].as_str().unwrap_or("");
        let name_b = b["perundingan"].as_str().unwrap_or("");
        
        val_b.partial_cmp(&val_a)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| name_a.cmp(name_b))
    });

    tw3_data.sort_by(|a, b| {
        let val_a = a["value"].as_f64().unwrap_or(0.0);
        let val_b = b["value"].as_f64().unwrap_or(0.0);
        let name_a = a["perundingan"].as_str().unwrap_or("");
        let name_b = b["perundingan"].as_str().unwrap_or("");
        
        val_b.partial_cmp(&val_a)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| name_a.cmp(name_b))
    });

    tw4_data.sort_by(|a, b| {
        let val_a = a["value"].as_f64().unwrap_or(0.0);
        let val_b = b["value"].as_f64().unwrap_or(0.0);
        let name_a = a["perundingan"].as_str().unwrap_or("");
        let name_b = b["perundingan"].as_str().unwrap_or("");
        
        val_b.partial_cmp(&val_a)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| name_a.cmp(name_b))
    });

    let result = vec![
        json!({"category": "TW1", "data": tw1_data}),
        json!({"category": "TW2", "data": tw2_data}),
        json!({"category": "TW3", "data": tw3_data}),
        json!({"category": "TW4", "data": tw4_data}),
    ];

    HttpResponse::Ok().json(result)
}