use chrono::{Datelike, Local};

fn main() {
    let mut current_period = (String::new(), String::new());

    let period_window = (9, 23);
    let now = Local::now().date_naive();
    let day = now.day();

    if day <= period_window.0 {
        current_period.0 = now
            .with_day(period_window.1 + 1)
            .unwrap()
            .with_month(if now.month() == 1 {
                12
            } else {
                now.month() - 1
            })
            .unwrap()
            .to_string();
        current_period.1 = now.with_day(period_window.0 - 1).unwrap().to_string();
    } else if day >= period_window.1 {
        current_period.0 = now.with_day(period_window.1 + 1).unwrap().to_string();
        current_period.1 = now
            .with_day(period_window.0 - 1)
            .unwrap()
            .with_month(if now.month() == 12 {
                1
            } else {
                now.month() + 1
            })
            .unwrap()
            .to_string();
    } else {
        current_period.0 = now.with_day(period_window.0).unwrap().to_string();
        current_period.1 = now.with_day(period_window.1).unwrap().to_string();
    }

//     info!("Current period: {:?}", current_period);
}
