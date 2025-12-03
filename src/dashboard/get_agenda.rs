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
            date,
            concat(time_start, ' - ', time_end) as time,
            perundingan,
            jenis,
            pembahasan
        FROM db_kerjasama.agenda a
        INNER JOIN (
        	SELECT
        		perundingan
        	FROM db_kerjasama.kerjasama
            WHERE TRUE
            {where_category}
            {where_information}
            {where_perundingan}
        ) b ON a.perundingan = b.perundingan
        {where_date}
        ORDER BY date ASC, time_start ASC
        FORMAT JSON;"
    );

    let res_query = query_ch(query).await.unwrap_or(Value::Null);
    let data = map_get("data", res_query)
        .as_array()
        .unwrap_or(&vec![])
        .to_owned();

    HttpResponse::Ok().json(data)
}