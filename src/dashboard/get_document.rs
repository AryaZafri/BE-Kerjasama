use actix_web::{HttpResponse, Responder, web};
use serde_json::{json, Value};

use crate::{map_get, query_ch};

pub async fn get_document(info: web::Json<Value>) -> impl Responder {
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

    let filters = map_get("filters", param.clone());
    let mut where_filters = String::new();

    if let Some(filters_obj) = filters.as_object() {
        let valid_columns = vec!["perundingan", "negara_mitra", "jenis", "last_update", "pembahasan", "timestamp"];
        
        for (column, value) in filters_obj {
            if valid_columns.contains(&column.as_str()) {
                if let Some(filter_value) = value.as_str() {
                    if filter_value != "" {
                        if column == "negara_mitra" || column == "pembahasan" || column == "perundingan" {
                            where_filters.push_str(&format!(
                                " AND {} ILIKE '%{}%'",
                                column,
                                filter_value.replace("'", "''")
                            ));
                        } else {
                            where_filters.push_str(&format!(
                                " AND {} = '{}'",
                                column,
                                filter_value.replace("'", "''")
                            ));
                        }
                    }
                }
            }
        }
    }

    let search = map_get("search", param.clone())
        .as_str()
        .unwrap_or("")
        .to_owned();

    let where_search = if search != "" {
        format!(
            "AND (perundingan ILIKE '%{0}%' OR negara_mitra ILIKE '%{0}%' OR jenis ILIKE '%{0}%' OR pembahasan ILIKE '%{0}%')",
            search.replace("'", "''")
        )
    } else {
        format!("")
    };

    let sorts = map_get("sorts", param.clone());
    let mut order_by_clauses = Vec::new();

    if let Some(sorts_array) = sorts.as_array() {
        let valid_columns = vec!["perundingan", "negara_mitra", "jenis", "last_update", "pembahasan", "timestamp"];
        
        for sort_item in sorts_array {
            if let Some(sort_obj) = sort_item.as_object() {
                let column = sort_obj.get("column")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                let direction = sort_obj.get("direction")
                    .and_then(|v| v.as_str())
                    .unwrap_or("ASC")
                    .to_uppercase();

                if valid_columns.contains(&column) && (direction == "ASC" || direction == "DESC") {
                    order_by_clauses.push(format!("{} {}", column, direction));
                }
            }
        }
    }

    let order_by = if order_by_clauses.is_empty() {
        "ORDER BY timestamp DESC".to_string()
    } else {
        format!("ORDER BY {}", order_by_clauses.join(", "))
    };

    let page = map_get("page", param.clone())
        .as_i64()
        .unwrap_or(1);
    
    let page_size = map_get("page_size", param.clone())
        .as_i64()
        .unwrap_or(10);

    let offset = (page - 1) * page_size;

    let limit_clause = if page > 0 && page_size > 0 {
        format!("LIMIT {} OFFSET {}", page_size, offset)
    } else {
        format!("")
    };

    let query = format!("
        SELECT
            perundingan,
            negara_mitra,
            jenis,
            last_update,
            pembahasan,
            document,
            timestamp
        FROM db_kerjasama.document a
        INNER JOIN (
            SELECT
                perundingan 
            FROM db_kerjasama.kerjasama
            WHERE TRUE
            {where_category}
            {where_information}
            {where_perundingan}
        ) b ON a.perundingan = b.perundingan
        WHERE TRUE
        {where_filters}
        {where_search}
        {order_by}
        {limit_clause}
        FORMAT JSON;"
    );

    let count_query = format!("
        SELECT
            COUNT(*) as total
        FROM db_kerjasama.document a
        INNER JOIN (
            SELECT
                perundingan 
            FROM db_kerjasama.kerjasama
            WHERE TRUE
            {where_category}
            {where_information}
            {where_perundingan}
        ) b ON a.perundingan = b.perundingan
        WHERE TRUE
        {where_filters}
        {where_search}
        FORMAT JSON;"
    );

    let res_query = query_ch(query).await.unwrap_or(Value::Null);
    let res_count = query_ch(count_query).await.unwrap_or(Value::Null);

    let total = map_get("data", res_count)
        .as_array()
        .and_then(|arr| arr.get(0))
        .and_then(|item| map_get("total", item.clone()).as_i64())
        .unwrap_or(0);

    let data = map_get("data", res_query)
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let mut new_item = item.clone();
            if let Some(obj) = new_item.as_object_mut() {
                let mut new_obj = serde_json::Map::new();
                new_obj.insert("no".to_string(), json!(offset + index as i64 + 1));
                
                for (key, value) in obj.iter() {
                    new_obj.insert(key.clone(), value.clone());
                }
                
                return json!(new_obj);
            }
            new_item
        })
        .collect::<Vec<Value>>();

    let response = json!({
        "data": data,
        "pagination": {
            "total": total,
            "page": page,
            "page_size": page_size,
            "total_pages": (total as f64 / page_size as f64).ceil() as i64
        }
    });

    HttpResponse::Ok().json(response)
}