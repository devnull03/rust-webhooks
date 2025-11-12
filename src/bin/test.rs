use std::{collections::HashMap, fs};

use mail_parser::MessageParser;
use regex::Regex;

#[derive(PartialEq, Eq, Hash, Debug)]
enum JobAlertSource {
    Linkedin,
    // Glassdoor,
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct ParsedJob {
    source: JobAlertSource,
    job_id: String,
    title: String,
    location: String,
    link: String,
}



fn main() {
    let raw_email_contents = fs::read("test_mail.txt").expect("File should be in the same dir");

    let parsed_email = MessageParser::default().parse(&raw_email_contents).unwrap();

    println!(
        "email from : {:?}, to: {:?} \n subject: {:?}",
        parsed_email.from().unwrap(),
        parsed_email.to().unwrap(),
        parsed_email.subject().unwrap()
    );

    println!("{:?}", parsed_email.body_text(0).unwrap());
    

    let mut jobs: HashMap<String, ParsedJob> = HashMap::new();

    if let Some(body) = parsed_email.body_text(0) {
        let re = Regex::new(
            r"(?<line_before>[^\n]+)\n<https://www\.linkedin\.com/comm/jobs/view/(?<job_id>\d{10})",
        )
        .unwrap();

        for caps in re.captures_iter(&body) {
            let line_before_url = caps["line_before"].trim().to_string();
            let job_id = caps["job_id"].to_string();
            let job_link = format!("https://www.linkedin.com/comm/jobs/view/{}", &job_id);

            if line_before_url.starts_with("[image:")
                || line_before_url.starts_with("Actively recruiting")
                || line_before_url.ends_with("connection")
                || line_before_url.is_empty()
            {
                continue;
            }

            // println!("Found LinkedIn job:");
            // println!("  Line before URL: {}", line_before_url);
            // println!("  Job ID: {}", job_id);
            // println!(
            //     "  Full URL: https://www.linkedin.com/comm/jobs/view/{}",
            //     job_id
            // );
            // println!();

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
                        if !(line_before_url.starts_with("*") && line_before_url.ends_with("*")) {
                            line_before_url
                        } else {
                            "".to_owned()
                        }
                    },
                    source: JobAlertSource::Linkedin,
                    job_id,
                },
            );
        }
    }

    println!("jobs found: {:?}", &jobs);
}
