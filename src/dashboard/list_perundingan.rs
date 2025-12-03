use actix_web::{HttpResponse, Responder};
use serde_json::{json, Value};

use crate::{map_get, query_ch};

pub async fn list_perundingan() -> impl Responder {
    let query = format!("
        SELECT
            DISTINCT perundingan
        FROM db_kerjasama.kerjasama
        ORDER BY perundingan ASC
        FORMAT JSON;"
    );

    let res_query = query_ch(query).await.unwrap_or(Value::Null);
    let raw_data = map_get("data", res_query)
        .as_array()
        .unwrap_or(&vec![])
        .to_owned();

    let data: Vec<String> = raw_data.iter()
        .filter_map(|item| {
            map_get("perundingan", item.clone())
                .as_str()
                .map(|s| s.to_string())
        })
        .collect();

    let response = json!({
        "data": data
    });

    HttpResponse::Ok().json(response)
}