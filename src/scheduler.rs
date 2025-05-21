use apalis::{layers::retry::RetryPolicy, prelude::*};
use apalis_cron::{CronStream, Schedule};
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};

use crate::{
    helpers::{email, job_checker},
    AppData,
};

#[derive(Clone)]
pub struct CronjobData {
    pub message: String,
    pub _app_data: Arc<AppData>,
}
impl CronjobData {
    async fn execute(&self, _item: Reminder) {
        println!("{} from CronjobData::execute()!", &self.message);
        let optum_jobs = job_checker::optum().await.unwrap();

        if optum_jobs.len() > 0 {
            println!("found jobs !!!");
            let _email_res = email::send_email(
                &self._app_data.resend,
                format!("{:?}", optum_jobs).as_str(),
                Some("FOUND AN OPTUM JOB !!!"),
            )
            .await
            .unwrap();
        } else {
            println!("no job :(");
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Reminder(DateTime<Utc>);
impl From<DateTime<Utc>> for Reminder {
    fn from(t: DateTime<Utc>) -> Self {
        Reminder(t)
    }
}

pub async fn say_hello_world(job: Reminder, svc: Data<CronjobData>) {
    // println!("Hello world from send_reminder()!");
    // this executes CronjobData::execute()
    svc.execute(job).await;
}

pub fn build_cron_worker_monitor(shared_state: Arc<AppData>) -> Monitor {
    // Format: sec min hour day_of_month month day_of_week (year)
    let schedule = Schedule::from_str("0 0 0 1-31/2 * *").expect("Couldn't start the scheduler!");
    let cron_service_ext = CronjobData {
        message: "Hello world".to_string(),
        _app_data: shared_state.clone(),
    };

    let monitor_instance = Monitor::new().register({
        WorkerBuilder::new("morning-cereal")
            .data(cron_service_ext)
            .retry(RetryPolicy::retries(5))
            .backend(CronStream::new_with_timezone(schedule, Local))
            .build_fn(say_hello_world)
    });
    monitor_instance
}
