use actix_web::{HttpResponse, Responder};
use serde_json::{json, Value};

use crate::{map_get, query_ch};

pub async fn list_year() -> impl Responder {
    let query = format!("
        SELECT DISTINCT year FROM (
            SELECT year FROM db_kerjasama.export
            UNION ALL
            SELECT year FROM db_kerjasama.import
        )
        ORDER BY year
        FORMAT JSON;"
    );

    let res_query = query_ch(query).await.unwrap_or(Value::Null);
    let raw_data = map_get("data", res_query)
        .as_array()
        .unwrap_or(&vec![])
        .to_owned();

    let data: Vec<String> = raw_data.iter()
        .filter_map(|item| {
            map_get("year", item.clone())
                .as_str()
                .map(|s| s.to_string())
        })
        .collect();

    let response = json!({
        "data": data
    });

    HttpResponse::Ok().json(response)
}