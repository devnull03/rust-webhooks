pub mod scheduler {
    use anyhow::Ok;
    use tracing::info;

    use crate::models::job::*;

    pub async fn optum() -> Result<Vec<optum::Job>, anyhow::Error> {
        info!("fetching optum jobs page api");
        let url = "https://jobsapi-internal.m-cloud.io/api/job?callback=CWS.jobs.jobCallback&facet[]=multi_select1:Technology&facet[]=level:Student Internships&facet[]=ats_portalid:Smashfly&latitude=28.4594965&longitude=77.0266383&LocationRadius=25&Limit=10&Organization=2071&offset=1&useBooleanKeywordSearch=true";
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

    pub async fn alert_email_handler(_from: &str, _email_content: &str) -> Result<(), ()> {


        
        Ok(())
    }
}
