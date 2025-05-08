use apalis::layers::retry::RetryPolicy;
use apalis::prelude::*;

use apalis_cron::CronContext;
use apalis_cron::CronStream;
use apalis_cron::Schedule;

use chrono::DateTime;
use chrono::Local;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use crate::AppData;

// #[derive(Debug, Default)]
// struct Reminder;

// async fn trigger_job_checker(_job: Reminder, ctx: CronContext<Local>) {
//     info!("Running cronjob for timestamp: {}", ctx.get_timestamp());
//     // Do something
//     println!("hello")
// }

#[derive(Clone)]
pub struct CronjobData {
    pub message: String,
    pub AppData: Arc<AppData>,
}
impl CronjobData {
    fn execute(&self, _item: Reminder) {
        println!("{} from CronjobData::execute()!", &self.message);
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
    println!("Hello world from send_reminder()!");
    // this executes CronjobData::execute()
    svc.execute(job);
}

pub fn build_cron_worker_monitor(shared_state: Arc<AppData>) -> Monitor {
    let schedule = Schedule::from_str("1 * * * * *").expect("Couldn't start the scheduler!");
    let cron_service_ext = CronjobData {
        message: "Hello world".to_string(),
        AppData: shared_state.clone(),
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
