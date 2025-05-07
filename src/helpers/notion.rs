use reqwest::{header, Client};
use std::error::Error;
use tracing::{info, error};

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

pub async fn fetch_data(client: &Client, db_id: &String) -> Result<structs::NotionResponse, Box<dyn Error>> {
    info!("Building filters for database query");
    let filters = utils::build_filters();

    let url = format!("https://api.notion.com/v1/databases/{db_id}/query");
    info!("Fetching data from Notion database: {}", db_id);
    
    let response = match client.post(&url).body(filters).send().await {
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
    
    let text = match response.text().await {
        Ok(text) => {
            info!("Successfully received response from Notion API");
            text
        },
        Err(e) => {
            error!("Failed to read response body: {}", e);
            return Err(Box::new(e));
        }
    };

    match serde_json::from_str::<structs::NotionResponse>(&text) {
        Ok(notion_data) => {
            info!("Successfully parsed Notion response with {} results", notion_data.results.len());
            Ok(notion_data)
        },
        Err(e) => {
            error!("Failed to parse Notion response: {}", e);
            error!("Raw response: {}", text);
            Err(Box::new(e))
        }
    }
}

pub async fn retrive_db(client: &reqwest::Client, db_id: &String) -> Result<String, Box<dyn Error>> {
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

pub mod utils {
    use chrono::{Datelike, Local, NaiveDate};
    use tracing::info;

    pub fn get_current_pay_period() -> (NaiveDate, NaiveDate) {
        let mut current_period: (NaiveDate, NaiveDate) =
            (NaiveDate::default(), NaiveDate::default());

        let period_window = (9, 23);
        let now = Local::now().date_naive();
        let day = now.day();
        
        info!("Calculating pay period for current date: {}", now);

        if day <= period_window.0 {
            current_period.0 = now
                .with_day(period_window.1 + 1)
                .unwrap()
                .with_month(if now.month() == 1 {
                    12
                } else {
                    now.month() - 1
                })
                .unwrap();
            current_period.1 = now.with_day(period_window.0 - 1).unwrap();
            info!("Period calculated (early month): {} to {}", current_period.0, current_period.1);
        } else if day >= period_window.1 {
            current_period.0 = now.with_day(period_window.1 + 1).unwrap();
            current_period.1 = now
                .with_day(period_window.0 - 1)
                .unwrap()
                .with_month(if now.month() == 12 {
                    1
                } else {
                    now.month() + 1
                })
                .unwrap();
            info!("Period calculated (late month): {} to {}", current_period.0, current_period.1);
        } else {
            current_period.0 = now.with_day(period_window.0).unwrap();
            current_period.1 = now.with_day(period_window.1).unwrap();
            info!("Period calculated (mid month): {} to {}", current_period.0, current_period.1);
        }

        current_period
    }

    pub fn build_filters() -> String {
        let date_property_name = "start and end";
        let current_pay_period = get_current_pay_period();
        
        info!("Building filters for pay period: {} to {}", 
            current_pay_period.0, 
            current_pay_period.1
        );

        let filter_string = format!(
            r#"{{"filter": {{"or": [ {{"property": "notes","rich_text": {{"contains": "\\ TODO"}}}},{{"and": [{{"property": "{date_property_name}","date": {{"on_or_after": "{pay_period_start}"}}}},{{"property": "{date_property_name}","date": {{"on_or_before": "{pay_period_end}"}}}} ]}} ]}}}}"#,
            pay_period_start = current_pay_period.0,
            pay_period_end = current_pay_period.1
        );

        info!("Filter string created with length: {} chars", filter_string.len());
        filter_string
    }
}

pub mod structs {
    use serde::{Deserialize, Serialize};
    use std::fmt;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct WebhookAutomationEvent {
        pub source: AutomationSource,
        pub data: serde_json::Value, // Using generic Value, don't really need this shit
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct AutomationSource {
        #[serde(rename = "type")]
        pub source_type: String,
        pub automation_id: String,
        pub action_id: String,
        pub event_id: Option<String>,
        pub user_id: Option<String>,
        pub attempt: Option<i32>,
    }

    // Response structs for Notion API
    #[derive(Serialize, Deserialize, Debug)]
    pub struct NotionResponse {
        object: String,
        pub results: Vec<Page>,
        next_cursor: Option<String>,
        has_more: bool,
    }

    impl fmt::Display for NotionResponse {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "Notion Response:")?;
            writeln!(f, "  Object Type: {}", self.object)?;
            writeln!(f, "  Has More: {}", self.has_more)?;
            writeln!(f, "  Next Cursor: {:?}", self.next_cursor)?;
            writeln!(f, "  Results Count: {}", self.results.len())?;

            for (i, page) in self.results.iter().enumerate() {
                writeln!(f, "\n=========== Page #{} ===========", i + 1)?;
                writeln!(f, "  ID: {}", page.id)?;
                writeln!(f, "  Object Type: {}", page.object)?;
                writeln!(f, "  URL: {}", page.url)?;
                writeln!(f, "  Created: {}", page.created_time)?;
                writeln!(f, "  Last Edited: {}", page.last_edited_time)?;

                writeln!(f, "\n  Properties:")?;

                // Start and End Date
                writeln!(
                    f,
                    "    Start and End (ID: {}):",
                    page.properties.start_and_end.id
                )?;
                writeln!(
                    f,
                    "      Type: {}",
                    page.properties.start_and_end.property_type
                )?;
                writeln!(
                    f,
                    "      Start: {}",
                    page.properties.start_and_end.date.start
                )?;
                writeln!(f, "      End: {:?}", page.properties.start_and_end.date.end)?;
                writeln!(
                    f,
                    "      Timezone: {:?}",
                    page.properties.start_and_end.date.time_zone
                )?;

                // Billable Hours
                writeln!(
                    f,
                    "    Billable Hours (ID: {}):",
                    page.properties.billable_hours.id
                )?;
                writeln!(
                    f,
                    "      Type: {}",
                    page.properties.billable_hours.property_type
                )?;
                writeln!(
                    f,
                    "      Formula Type: {}",
                    page.properties.billable_hours.formula.value_type
                )?;
                writeln!(
                    f,
                    "      Hours: {:?}",
                    page.properties.billable_hours.formula.number
                )?;

                // Workplace
                writeln!(f, "    Workplace (ID: {}):", page.properties.workplace.id)?;
                writeln!(f, "      Type: {}", page.properties.workplace.property_type)?;
                writeln!(
                    f,
                    "      Select ID: {}",
                    page.properties.workplace.select.id
                )?;
                writeln!(f, "      Name: {}", page.properties.workplace.select.name)?;
                writeln!(f, "      Color: {}", page.properties.workplace.select.color)?;

                // Duration
                writeln!(f, "    Duration (ID: {}):", page.properties.duration.id)?;
                writeln!(f, "      Type: {}", page.properties.duration.property_type)?;
                writeln!(
                    f,
                    "      Formula Type: {}",
                    page.properties.duration.formula.value_type
                )?;
                writeln!(
                    f,
                    "      Value: {:?}",
                    page.properties.duration.formula.number
                )?;

                // Notes
                writeln!(f, "    Notes (ID: {}):", page.properties.notes.id)?;
                writeln!(f, "      Type: {}", page.properties.notes.property_type)?;
                writeln!(
                    f,
                    "      Text Count: {}",
                    page.properties.notes.rich_text.len()
                )?;

                for (j, text) in page.properties.notes.rich_text.iter().enumerate() {
                    writeln!(f, "      Text #{}", j + 1)?;
                    writeln!(f, "        Type: {}", text.text_type)?;
                    writeln!(f, "        Content: {}", text.text.content)?;
                    writeln!(f, "        Plain Text: {}", text.plain_text)?;
                    writeln!(f, "        Href: {:?}", text.href)?;
                }
            }
            Ok(())
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Page {
        object: String,
        id: String,
        created_time: String,
        last_edited_time: String,
        pub properties: PageProperties,
        url: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct PageProperties {
        #[serde(rename = "start and end")]
        pub start_and_end: DateProperty,
        #[serde(rename = "Billable Hours")]
        pub billable_hours: FormulaProperty,
        #[serde(rename = "Workplace")]
        workplace: SelectProperty,
        #[serde(rename = "Duration")]
        duration: FormulaProperty,
        #[serde(rename = "notes")]
        notes: RichTextProperty,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DateProperty {
        id: String,
        #[serde(rename = "type")]
        property_type: String,
        pub date: DateValue,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DateValue {
        pub start: String,
        pub end: Option<String>,
        time_zone: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct FormulaProperty {
        id: String,
        #[serde(rename = "type")]
        property_type: String,
        pub formula: FormulaValue,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct FormulaValue {
        #[serde(rename = "type")]
        value_type: String,
        pub number: Option<f64>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct SelectProperty {
        id: String,
        #[serde(rename = "type")]
        property_type: String,
        select: SelectValue,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct SelectValue {
        id: String,
        name: String,
        color: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RichTextProperty {
        id: String,
        #[serde(rename = "type")]
        property_type: String,
        rich_text: Vec<RichTextValue>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RichTextValue {
        #[serde(rename = "type")]
        text_type: String,
        text: TextContent,
        annotations: Option<serde_json::Value>,
        plain_text: String,
        href: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct TextContent {
        content: String,
        link: Option<serde_json::Value>,
    }
}
