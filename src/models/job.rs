pub mod optum {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct JobResponse {
        pub aggregations: Option<serde_json::Value>,
        pub titles: Option<serde_json::Value>,
        #[serde(rename = "totalHits")]
        pub total_hits: u32,
        #[serde(rename = "queryResult")]
        pub query_result: Vec<Job>,

    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Job {
        pub company_name: String,
        pub id: u64,
        pub industry: String,
        pub title: String,
        pub primary_city: String,
        pub level: String,
        pub easy_apply: Vec<serde_json::Value>,
        pub internal_url: String,
        pub internal_description: String,
        // All other fields will be ignored by serde
        #[serde(flatten)]
        pub _other: serde_json::Value,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AdditionalLocation {
        pub addtnl_city: String,
        pub addtnl_state: String,
        pub addtnl_zip: String,
        pub addtnl_country: String,
        pub addtnl_address: String,
        pub addtnl_location: Vec<f64>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct JobClassification {
        pub major_code: String,
        pub major_description: String,
        pub major_score: f64,
        pub minor_code: String,
        pub minor_description: String,
        pub minor_score: f64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CustomFields {
        pub req_custom_field_3: Option<CustomField>,
        // Add other custom fields as needed
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CustomField {
        pub label: String,
        pub value: String,
    }
}
