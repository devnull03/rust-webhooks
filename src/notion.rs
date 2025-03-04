use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

pub fn notion_client_init() -> Client {
    let mut notion_api_key = header::HeaderValue::from_str(
        format!("Bearer {}", env::var("NOTION_API_KEY").unwrap()).as_str(),
    )
    .unwrap();
    notion_api_key.set_sensitive(true);

    let mut headers = header::HeaderMap::new();
    headers.insert(header::AUTHORIZATION, notion_api_key);
    headers.insert(
        "Notion-Version",
        header::HeaderValue::from_static("2022-06-28"),
    );
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );

    let client = Client::builder().default_headers(headers).build().unwrap();
    client
}

fn get_current_pay_period() -> (String, String) {
    ("2025-02-24".to_string(), "2025-03-08".to_string())
}

fn build_filters() -> String {
    let date_property_name = "start and end";
    let current_pay_period = get_current_pay_period();

    let filter_string = format!(
        "{{
		\"filter\": {{
			\"and\": [
				{{
					\"property\": \"{date_property_name}\",
					\"date\": {{
						\"on_or_after\": \"{pay_period_start}\"
					}}
				}},
				{{
					\"Property\": \"{date_property_name}\",
					\"Date\": {{
						\"on_or_before\": \"{pay_period_end}\"
					}}
				}},
			]
		}}
	}}",
        pay_period_start = current_pay_period.0,
        pay_period_end = current_pay_period.1
    );

    filter_string
}

#[derive(Serialize, Deserialize)]
struct QueryResponse {
    object: String,
    next_cursor: String,
    has_more: bool,
    results: Vec<Value> 
}

struct FilterResultProperties {

}

pub async fn fetch_data(client: &Client) {
    let url = format!(
        "https://api.notion.com/v1/databases/{db_id}/query",
        db_id = env::var("DB_ID").unwrap()
    );
    let filters = build_filters();

    let res = client.post(url).body(filters).send().await.unwrap();


}
