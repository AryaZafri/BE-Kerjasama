use actix_web::{HttpResponse, Responder, web};
use serde_json::{json, Value};

use crate::{map_get, query_ch};

pub async fn kerjasama_proses_berlaku(info: web::Json<Value>) -> impl Responder {
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
        format!("AND information = '{}'", information.replace("'", "''"))
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
            information,
            jumlah,
            ROUND((jumlah * 100.0) / SUM(jumlah) OVER (), 2) AS percentage
        FROM (
            SELECT
                multiIf(`year` != '', 'Berlaku', 'Proses') AS information,
                COUNT(*) AS jumlah
            FROM db_kerjasama.kerjasama
            WHERE TRUE
            {where_category}
            {where_information}
            {where_perundingan}
            GROUP BY information
        )
        ORDER BY information ASC
        FORMAT JSON;"
    );

    let res_query = query_ch(query).await.unwrap_or(Value::Null);
    let data = map_get("data", res_query)
        .as_array()
        .unwrap_or(&vec![])
        .to_owned();

    let total: f64 = data.iter()
        .map(|item| {
            map_get("jumlah", item.clone())
                .as_u64()
                .unwrap_or(0) as f64
        })
        .sum();

    let result: Vec<Value> = data.iter()
        .map(|item| {
            let jumlah = map_get("jumlah", item.clone())
                .as_u64()
                .unwrap_or(0);
            let information = map_get("information", item.clone())
                .as_str()
                .unwrap_or("")
                .to_owned();
            
            let percentage = if total > 0.0 {
                ((jumlah as f64 / total) * 100.0 * 100.0).round() / 100.0
            } else {
                0.0
            };

            json!({
                "information": information,
                "jumlah": jumlah,
                "percentage": percentage
            })
        })
        .collect();

    HttpResponse::Ok().json(result)
}