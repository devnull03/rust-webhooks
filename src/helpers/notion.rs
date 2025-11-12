use reqwest::{header, Client};
use std::error::Error;
use tracing::{info, error};

// use crate::models;

pub fn notion_client_init(key: String) -> Result<Client, Box<dyn Error>> {
    info!("Initializing Notion client");
    
    let notion_api_key = match header::HeaderValue::from_str(format!("Bearer {}", key).as_str()) {
        Ok(value) => {
            let mut val = value;
            val.set_sensitive(true);
            val
        },
        Err(e) => {
            error!("Failed to create Authorization header value: {}", e);
            return Err(Box::new(e));
        }
    };

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

    info!("Building Notion client with headers");
    match Client::builder().default_headers(headers).build() {
        Ok(client) => {
            info!("Notion client initialized successfully");
            Ok(client)
        },
        Err(e) => {
            error!("Failed to build Notion client: {}", e);
            Err(Box::new(e))
        }
    }
}

// pub async fn fetch_data(client: &Client, db_id: &String) -> Result<models::notion::NotionResponse, Box<dyn Error>> {
//     info!("Building filters for database query");
//     // let filters = utils::build_filters();

//     let url = format!("https://api.notion.com/v1/databases/{db_id}/query");
//     info!("Fetching data from Notion database: {}", db_id);
    
//     let response = match client.post(&url).body("").send().await {
//         Ok(resp) => {
//             if !resp.status().is_success() {
//                 let status = resp.status();
//                 let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
//                 error!("Notion API returned error status {}: {}", status, error_text);
//                 return Err(format!("Notion API returned status {}: {}", status, error_text).into());
//             }
//             resp
//         },
//         Err(e) => {
//             error!("Failed to send request to Notion API: {}", e);
//             return Err(Box::new(e));
//         }
//     };
    
//     let text = match response.text().await {
//         Ok(text) => {
//             info!("Successfully received response from Notion API");
//             text
//         },
//         Err(e) => {
//             error!("Failed to read response body: {}", e);
//             return Err(Box::new(e));
//         }
//     };

//     match serde_json::from_str::<models::notion::NotionResponse>(&text) {
//         Ok(notion_data) => {
//             info!("Successfully parsed Notion response with {} results", notion_data.results.len());
//             Ok(notion_data)
//         },
//         Err(e) => {
//             error!("Failed to parse Notion response: {}", e);
//             error!("Raw response: {}", text);
//             Err(Box::new(e))
//         }
//     }
// }

pub async fn _retrive_db(client: &reqwest::Client, db_id: &String) -> Result<String, Box<dyn Error>> {
    let url = format!("https://api.notion.com/v1/databases/{db_id}/");
    info!("Retrieving database structure from: {}", url);
    
    let response = match client.get(&url).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                error!("Notion API returned error status {}: {}", status, error_text);
                return Err(format!("Notion API returned status {}: {}", status, error_text).into());
            }
            resp
        },
        Err(e) => {
            error!("Failed to send request to Notion API: {}", e);
            return Err(Box::new(e));
        }
    };
    
    match response.text().await {
        Ok(text) => {
            info!("Successfully retrieved database structure, response length: {} chars", text.len());
            Ok(format!("{:?}", &text))
        },
        Err(e) => {
            error!("Failed to read response body: {}", e);
            Err(Box::new(e))
        }
    }
}
