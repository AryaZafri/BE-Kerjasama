use serde_json::Value;
use serde_merge::Map;
use std::{str::FromStr, time::{UNIX_EPOCH, SystemTime}};
use chrono::{Duration, NaiveDate, NaiveDateTime, Datelike};
use std::collections::BTreeMap;
use reqwest::{Client, Result, Error};

use crate::{get_client, debug::log};

pub fn phonenumberto62(phonenumber: String) -> String {
    let mut deleteplus = phonenumber.replace("+", "");

    if deleteplus.starts_with("0") {
        deleteplus.replace_range(0..1, "62")
    };

    deleteplus
}

pub fn week_to_date_range(week_str: &str) -> Option<(NaiveDate, NaiveDate)> {
    if week_str.len() == 7 {
        let year = i32::from_str(&week_str[0..4]).ok()?;
        let week = u32::from_str(&week_str[5..7]).ok()?;

        let jan_1 = NaiveDate::from_ymd_opt(year, 1, 1)?;

        let jan_1_weekday = jan_1.weekday().num_days_from_monday();
        let days_to_week_start = (week - 1) * 7;
        let start_date = jan_1 + Duration::days(days_to_week_start as i64 - jan_1_weekday as i64);

        let end_date = start_date + Duration::days(6);

        Some((start_date, end_date))
    } else {
        None
    }
}

pub fn month_to_date_range(month_str: &str) -> Option<(NaiveDate, NaiveDate)> {
    if month_str.len() == 7 {
        let year = i32::from_str(&month_str[0..4]).ok()?;
        let month = u32::from_str(&month_str[5..7]).ok()?;

        let start_date = NaiveDate::from_ymd_opt(year, month, 1)?;

        let end_date = start_date
            .with_day(1)
            .and_then(|d| d.checked_add_signed(Duration::days(32)))
            .and_then(|d| d.with_day(1))
            .and_then(|d| d.checked_sub_signed(Duration::days(1)));

        end_date.map(|end| (start_date, end))
    } else {
        None
    }
}

pub fn year_to_date_range(year_str: &str) -> Option<(NaiveDate, NaiveDate)> {
    if year_str.len() == 4 {
        let year = i32::from_str(year_str).ok()?;

        let start_date = NaiveDate::from_ymd_opt(year, 1, 1)?;

        let end_date = NaiveDate::from_ymd_opt(year, 12, 31)?;

        Some((start_date, end_date))
    } else {
        None
    }
}

pub fn calculate_days_between(start_date: &str, end_date: &str) -> i64 {
    let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d").expect("Invalid start_date format");
    let end = NaiveDate::parse_from_str(end_date, "%Y-%m-%d").expect("Invalid end_date format");
    (end - start).num_days()
}

pub fn map_get_value(key: &str, value: &BTreeMap<String, Value>) -> Value {
    value
        .get(key)
        .unwrap_or(&Value::Null)
        .to_owned()
}

pub fn map_get_str(key: &str, value: &BTreeMap<String, Value>) -> String {
    value
        .get(key)
        .unwrap_or(&Value::Null)
        .as_str()
        .unwrap_or(&"").to_owned()
}

pub fn map_get_f64(key: &str, value: &BTreeMap<String, Value>) -> f64 {
    value
        .get(key)
        .unwrap_or(&Value::Null)
        .as_f64()
        .unwrap_or(0.0).to_owned()
}

pub fn map_get_i64(key: &str, value: &BTreeMap<String, Value>) -> i64 {
    value
        .get(key)
        .unwrap_or(&Value::Null)
        .as_i64()
        .unwrap_or(0).to_owned()
}

pub fn map_get_vec_value(key: &str, value: &BTreeMap<String, Value>) -> Vec<Value> {
    value
        .get(key)
        .unwrap_or(&Value::Null)
        .as_array()
        .unwrap_or(&Vec::new()).to_owned()
}

pub fn map_get(key: &str, value: Value) -> Value {
    value
        .as_object()
        .unwrap_or(&Map::new())
        .get(key)
        .unwrap_or(&Value::Null)
        .to_owned()
}

pub fn object_get(key: &str, value: Map) -> Value {
    value
        .get(key)
        .unwrap_or(&Value::Null)
        .to_owned()
}

pub fn to_array(map: Value) -> Vec<Value> {
    map.as_array().unwrap_or(&Vec::new()).to_owned()
}

pub fn to_object(map: Value) -> Map {
    map.as_object().unwrap_or(&Map::new()).to_owned()
}

pub fn to_f64(map: Value) -> f64 {
    map.as_f64().unwrap_or(0.0).to_owned()
}

pub fn to_i64(v: Value) -> i64 {
    match v {
        Value::Number(num) => num.as_i64().unwrap_or(0),
        Value::String(s) => s.parse::<i64>().unwrap_or(0),
        _ => 0,
    }
}

pub fn to_str(map: Value) -> String {
    map.as_str().unwrap_or("").to_owned()
}

pub fn get_first(vec: Vec<Value>) -> Value  {
    vec.first().unwrap_or(&Value::Null).to_owned()
}

pub async fn get_result(data: Value) -> Value {
        Value::from_str(map_get("results", data).as_str().unwrap_or("")).unwrap_or(Value::Null)
}

pub async fn convert_datetime(date: Value) -> i64 {
    let date_str = date.as_str().unwrap_or("");
    let naive_date_time = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
        .unwrap_or(NaiveDateTime::default())
        .and_utc()
        .timestamp();

    naive_date_time
}

pub fn map_filter_equal_to(data: Map<>, match_value: Value) -> Value {
    get_first(data.iter().filter(|(x, _y)| {
        x.to_string() == to_str(match_value.to_owned())
    })
    .map(|(_x,y)| y.to_owned())
    .collect::<Vec<_>>())
}

pub fn vec_filter_equal_to(data: Vec<Value>, key: &str, match_value: Value) -> Map<> {
    data.into_iter()
        .filter(|x| {
            let map = map_get(key, x.clone());
            map.to_string().to_lowercase() == match_value.to_string().to_lowercase()
        })
        .collect::<Vec<Value>>()
        .clone()
        .get(0)
        .unwrap_or(&Value::Null)
        .as_object()
        .unwrap_or(&Map::new())
        .to_owned()
}

pub fn vec_filter_contains(data: Vec<Value>, key: &str, contains_value: Vec<Value>) -> Vec<Value> {
    data.into_iter()
        .filter(|x| {
            let map = map_get(key, x.clone());
            contains_value.contains(&map)
        })
        .collect::<Vec<Value>>()
        .clone()
}

pub fn clear_time_30min() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let in_ms = (since_the_epoch.as_secs() * 1000 +
          since_the_epoch.subsec_nanos() as u64 / 1_000_000) - (30 * 1000 * 60);
  
    println!("in ms: {:?}", in_ms);

    in_ms
  }

  pub fn clear_time_24h() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let in_ms = (since_the_epoch.as_secs() * 1000 +
          since_the_epoch.subsec_nanos() as u64 / 1_000_000) - (24 * 60 * 1000 * 60);
  
    println!("in ms: {:?}", in_ms);

    in_ms
  }

pub fn format_currency (num: String) -> String {
    let length = num.len();
    
    if length <= 3 {

        format!("{}", num)

    } else if length > 3 && length <= 6 {

        let (last, first) = num.split_at(length -3 );
        format!("{},{}", last, first)

    } else if length > 6 && length <= 9 {

        let (splt, first) = num.split_at(length -3 );
        let (last, second) = splt.split_at(splt.len() -3 );
        format!("{},{},{}", last, second, first)

    } else if length > 9 && length <= 12 {

        let (splt, first) = num.split_at(length -3 );
        let (splt_2, second) = splt.split_at(splt.len() -3 );
        let (last, third) = splt_2.split_at(splt_2.len() -3 );
        format!("{},{},{},{}", last, third, second, first)

    } else if length > 12 && length <= 15 {

        let (splt, first) = num.split_at(length -3 );
        let (splt_2, second) = splt.split_at(splt.len() -3 );
        let (splt_3, third) = splt_2.split_at(splt_2.len() -3 );
        let (last, fourth) = splt_3.split_at(splt_3.len() -3 );
        format!("{},{},{},{},{}", last, fourth, third, second, first)

    } else if length > 15 && length <= 18 {

        let (splt, first) = num.split_at(length -3 );
        let (splt_2, second) = splt.split_at(splt.len() -3 );
        let (splt_3, third) = splt_2.split_at(splt_2.len() -3 );
        let (splt_4, fourth) = splt_3.split_at(splt_3.len() -3 );
        let (last, five) = splt_4.split_at(splt_4.len() -3 );
        format!("{},{},{},{},{},{}", last, five, fourth, third, second, first)

    } else if length > 18 && length <= 21 {

        let (splt, first) = num.split_at(length -3 );
        let (splt_2, second) = splt.split_at(splt.len() -3 );
        let (splt_3, third) = splt_2.split_at(splt_2.len() -3 );
        let (splt_4, fourth) = splt_3.split_at(splt_3.len() -3 );
        let (splt_5, five) = splt_4.split_at(splt_4.len() -3 );
        let (last, six) = splt_5.split_at(splt_5.len() -3 );
        format!("{},{},{},{},{},{},{}", last, six, five, fourth, third, second, first)

    } else {
        num.to_string()
    }
}

pub async fn get_api_with_token(client: Result<Client>, url: &str, json_param: Value, token: String) -> anyhow::Result<Value, Error> {
    log(format!("url: {}, json_param: {}, token: {}", url, json_param.to_string(), token));
    let token2 = token.as_str();
    let res = client?
        .post(url)
        .header("accessToken", token2)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&json_param)
        .send()
        .await?;

    println!("status: {:?}", res.status());

    res.json::<Value>().await
}

pub async fn get_token_env() -> String {
    let env_status = std::env::var("ENV_STATUS").expect("no environment variable set for \"ENV STATUS\"");
    let token = if env_status == format!("DEPLOY") {
        get_token_from_txt() //deploy
    } else {
        get_token_local().await //lokal
    };

    token
}

pub fn get_token_from_txt() -> String {
    let token_path = std::env::var("TOKEN_PATH").expect("Fail to get token path!");
    let content = std::fs::read_to_string(token_path).expect("Failed to read token.txt!");
    let data = Value::from_str(content.as_str()).unwrap_or(Value::Null);
    
    map_get("AccessToken", data)
        .as_str()
        .unwrap_or("")
        .to_string()
}

pub async fn get_token_local() -> String {
    match get_request(
        get_client(),
        std::env::var("TOKEN_URL_LOCAL")
            .expect("Url Tidak ditemukan di .env!")
            .as_str(),
    ).await {
        Ok(obj) => {
            if let Some(value) = obj.as_object() {
                if let Some(key) = value.get("AccessToken") {key.as_str().unwrap_or("").to_string()} 
                else {"".to_string()}
            } else {"".to_string()}
        },
        Err(err) => {
            println!("Error: {:?}", err);
            "".to_string()
        }
    }
}

pub async fn get_request(client: Result<Client>, url: &str) -> anyhow::Result<Value> {
    let res = client?
        .get(url)
        .header("Content-Type", "application/json")
        .send()
        .await?;
    println!("Headers: {:?}", res.headers());
    let json_subscribe = res.json::<Value>().await?;

    Ok(json_subscribe)
}

pub async fn get_api_with_basic_adc(client: Result<Client>, url: &str, json_param: Value) -> anyhow::Result<Value, Error> {
    log(format!("url: {}, json_param: {}", url, json_param.to_string()));
    let res = client?
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", "Basic MTAwMnxUak0xak9mekw4NnplOUVHNDJhMDp3fn1yeFU/O1RfYjN6d19iV0lsSmxaNCZRO1NCdSpYVSE2cGVmI0c5")
        .body(json_param.to_string())
        .send() 
        .await?;
        
    println!("reponse: {:?}", res.headers());
    let json_subscribe = res.json::<Value>().await;

    json_subscribe
}
