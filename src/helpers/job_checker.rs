pub mod scheduler {
    use anyhow::Ok;
    use tracing::info;

    use crate::models::job::*;

    pub async fn optum() -> Result<Vec<optum::Job>, anyhow::Error> {
        info!("fetching optum jobs page api");
        // let url = "https://jobsapi-internal.m-cloud.io/api/job?callback=CWS.jobs.jobCallback&facet[]=multi_select1:Technology&facet[]=level:Student Internships&facet[]=ats_portalid:Smashfly&latitude=28.4594965&longitude=77.0266383&LocationRadius=25&Limit=10&Organization=2071&offset=1&useBooleanKeywordSearch=true";
        let url = "https://jobsapi-internal.m-cloud.io/api/job?callback=CWS.jobs.jobCallback&facet[]=parent_category:Optum&facet[]=level:Student Internships&facet[]=ats_portalid:Smashfly&latitude=28.4594965&longitude=77.0266383&LocationRadius=50&Limit=10&Organization=2071&offset=1&useBooleanKeywordSearch=true";
        let body = reqwest::get(url).await?.text().await?;

        let clean_body = body
            .strip_prefix("CWS.jobs.jobCallback(")
            .unwrap()
            .strip_suffix(")")
            .unwrap()
            .to_string();

        info!("parsed results from optum");
        let job_response: optum::JobResponse = serde_json::from_str(&clean_body)?;

        Ok(job_response.query_result)
        // Ok(serde_json::to_string(&job_response)?)
    }
}

pub mod server {
    use std::{any, collections::HashMap};

    use mail_parser::{Message, MessageParser};
    use regex::Regex;

    use crate::models::job::{JobAlertSource, ParsedJob};

    pub async fn alert_email_handler(_from: &str, email_content: &[u8]) -> Result<(), ()> {
        let parsed_email = MessageParser::default().parse(&email_content).unwrap();

        println!(
            "email from : {:?}, to: {:?} \n subject: {:?}",
            parsed_email.from().unwrap(),
            parsed_email.to().unwrap(),
            parsed_email.subject().unwrap()
        );

        if let Some(body) = parsed_email.body_text(0) {
            let re = Regex::new(r"jobalerts-noreply@linkedin\.com").unwrap();
            if re.is_match(&body) {
                println!("linkedin email found")
            }
        }

        Ok(())
    }

    pub fn parse_linkedin_email(parsed_email: Message) -> Result<HashMap<String, ParsedJob>, String> {
        let mut jobs: HashMap<String, ParsedJob> = HashMap::new();

        if let Some(body) = parsed_email.body_text(0) {
            let re = Regex::new(
            r"(?<line_before>[^\n]+)\n<https://www\.linkedin\.com/comm/jobs/view/(?<job_id>\d{10})",
        )
        .unwrap();

            for caps in re.captures_iter(&body) {
                let line_before_url = (&caps["line_before"]).trim().to_string();
                let job_id = caps["job_id"].to_string();
                let job_link = format!("https://www.linkedin.com/comm/jobs/view/{}", &job_id);

                if line_before_url.starts_with("[image:")
                    || line_before_url.starts_with("Actively recruiting")
                    || line_before_url.ends_with("connection")
                    || line_before_url.is_empty()
                {
                    continue;
                }

                println!("Found LinkedIn job:");
                println!("  Line before URL: {}", line_before_url);
                println!("  Job ID: {}", job_id);
                println!(
                    "  Full URL: https://www.linkedin.com/comm/jobs/view/{}",
                    job_id
                );
                println!();

                jobs.insert(
                    job_id.clone(),
                    ParsedJob {
                        link: job_link,
                        title: {
                            if line_before_url.starts_with("*") && line_before_url.ends_with("*") {
                                line_before_url.clone()
                            } else {
                                jobs.get(&job_id).unwrap().title.clone()
                            }
                        },
                        location: {
                            if !(line_before_url.starts_with("*") && line_before_url.ends_with("*"))
                            {
                                line_before_url
                            } else {
                                "".to_owned()
                            }
                        },
                        source: JobAlertSource::Linkedin,
                        job_id: job_id,
                    },
                );
            }
        }

        // println!("jobs found: {:?}", &jobs);

        Ok(jobs)
    }
}
