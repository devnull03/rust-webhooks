use apalis::{layers::retry::RetryPolicy, prelude::*};
use apalis_cron::{CronStream, Schedule};
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};

use crate::{helpers::job_checker, AppData};

#[derive(Clone)]
pub struct CronjobData {
    pub message: String,
    pub _app_data: Arc<AppData>,
}
impl CronjobData {
    fn execute(&self, _item: Reminder) {
        println!("{} from CronjobData::execute()!", &self.message);

        // job_checker::optum();
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
    svc.execute(job);
}

pub fn build_cron_worker_monitor(shared_state: Arc<AppData>) -> Monitor {
    let schedule = Schedule::from_str("1 * * * * *").expect("Couldn't start the scheduler!");
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
