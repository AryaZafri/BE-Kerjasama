use actix_web::{HttpResponse, Responder, web};
use serde_json::Value;

use crate::{map_get, query_ch};

pub async fn get_agenda(info: web::Json<Value>) -> impl Responder {
    let param = info.into_inner();
    
    let date = map_get("date", param.clone())
        .as_str()
        .unwrap_or("")
        .to_owned();

    let where_date = if date != "" {
        format!("WHERE date = '{}'", date.replace("'", "''"))
    } else {
        format!("--")
    };
    
    let query = format!(
        "SELECT
            toString(date) as date,
            time_start,
            time_end,
            perundingan,
            jenis,
            pembahasan,
            updated_by,
            toString(timestamp) as timestamp
        FROM kerjasama.agenda
        {where_date}
        ORDER BY date ASC, time_start ASC
        FORMAT JSON;"
    );
    println!("🔍 Query: {}", query);

    let res_query = query_ch(query).await.unwrap_or(Value::Null);
    let data = map_get("data", res_query)
        .as_array()
        .unwrap_or(&vec![])
        .to_owned();

    HttpResponse::Ok().json(data)
}