use actix_web::{HttpResponse, Responder, web};
use serde_json::{json, Value};

use crate::{map_get, query_ch};

pub async fn top_three_category(info: web::Json<Value>) -> impl Responder {
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
        WITH ranked_data AS (
            SELECT
                a.perundingan AS perundingan,
                a.category,
                b.jenis,
                CASE 
                    WHEN greatest(b.last_update, c.date) = toDateTime(0)
                        OR greatest(b.last_update, c.date) IS NULL
                    THEN '-'
                    ELSE toString(greatest(b.last_update, c.date))
                END AS last_update_final,
                ROW_NUMBER() OVER (
                    PARTITION BY a.category
                    ORDER BY greatest(b.last_update, c.date) DESC, a.perundingan ASC
                ) AS rn
            FROM db_kerjasama.kerjasama a
            LEFT JOIN (
                SELECT
                    perundingan,
                    jenis,
                    last_update
                FROM db_kerjasama.document
            ) b ON a.perundingan = b.perundingan
            LEFT JOIN (
                SELECT
                    date,
                    perundingan
                FROM db_kerjasama.agenda
            ) c ON a.perundingan = c.perundingan
            WHERE TRUE
            {where_category}
            {where_information}
            {where_perundingan}
        )
        SELECT
            category,
            perundingan,
            jenis,
            last_update_final AS last_update
        FROM ranked_data
        WHERE rn <= 3
        ORDER BY
            category ASC,
            last_update DESC,
            perundingan ASC
        FORMAT JSON;"
    );

    let res_query = query_ch(query).await.unwrap_or(Value::Null);
    let data = map_get("data", res_query)
        .as_array()
        .unwrap_or(&vec![])
        .to_owned();

    let categories = vec!["Regional", "Bilateral", "Multilateral"];
    
    let mut grouped: std::collections::HashMap<String, Vec<Value>> = std::collections::HashMap::new();
    
    for cat in categories.iter() {
        grouped.insert(cat.to_string(), Vec::new());
    }
    
    for item in data.iter() {
        let cat = map_get("category", item.clone())
            .as_str()
            .unwrap_or("")
            .to_owned();
        
        if categories.contains(&cat.as_str()) {
            if let Some(items) = grouped.get_mut(&cat) {
                let mut new_item = item.clone();
                if let Some(obj) = new_item.as_object_mut() {
                    obj.remove("category");
                }
                items.push(new_item);
            }
        }
    }

    let result: Vec<Value> = categories.into_iter().map(|category| {
        json!({
            "category": category,
            "data": grouped.get(category).unwrap_or(&Vec::new()).clone()
        })
    }).collect();

    HttpResponse::Ok().json(result)
}