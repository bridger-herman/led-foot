use std::fs::File;
use std::io::prelude::*;

use chrono::prelude::*;
use serde_json;

const SCHEDULE_FILE: &str = "schedules/schedule.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LedAlarm {
    days: Vec<String>,
    hour: String,
    minute: String,
    sequence: String,
}

#[derive(Debug, Deserialize)]
pub struct LedScheduler {
    pub alarms: Vec<LedAlarm>,
    current_active: Option<LedAlarm>,
}

impl LedScheduler {
    pub fn reset_alarms(&mut self, new_alarms: &[LedAlarm]) {
        let new_alarms: Vec<_> = new_alarms
            .iter()
            .map(|alarm| {
                let mut new_alarm = alarm.clone();
                new_alarm.hour =
                    format!("{:02?}", new_alarm.hour.parse::<u8>().unwrap());
                new_alarm.minute =
                    format!("{:02?}", new_alarm.minute.parse::<u8>().unwrap());
                new_alarm
            })
            .collect();
        self.alarms = new_alarms.to_vec();
    }

    pub fn rewrite_schedule(&self) {
        let mut file =
            File::create(SCHEDULE_FILE).expect("Unable to open schedule file");

        let json_string = serde_json::to_string_pretty(&self.alarms)
            .expect("Unable to encode schdeule json string");
        file.write_all(json_string.as_bytes())
            .expect("Unable to rewrite schedule");
    }

    pub fn one_frame(&mut self) {
        let now = Local::now();

        let now_weekday = &format!("{:02?}", now.weekday());
        let now_hour = &format!("{:02?}", now.hour());
        let now_minute = &format!("{:02?}", now.minute());

        let reset_active =
            if let Some(LedAlarm { ref minute, .. }) = self.current_active {
                minute != now_minute
            } else {
                false
            };

        if self.current_active.is_none() {
            for alarm in &self.alarms {
                for day in &alarm.days {
                    trace!(
                        "{:?} {:?} {:?} == {:?} {:?} {:?}",
                        now_weekday,
                        now_hour,
                        now_minute,
                        day,
                        &alarm.hour,
                        &alarm.minute
                    );
                    if now_weekday == day
                        && now_hour == &alarm.hour
                        && now_minute == &alarm.minute
                    {
                        info!(
                            "Starting on schedule: {} {}:{}",
                            day, alarm.hour, alarm.minute
                        );
                        {
                            let mut state = led_state!();
                            let active = state.active();
                            state.set_changed_from_ui(active);
                        }
                        led_system!().update_sequence(&alarm.sequence);
                        led_system!().run_sequence();
                        led_state!().set_changed_from_ui(false);
                        self.current_active = Some(alarm.clone());
                    }
                }
            }
        } else {
            debug!("Not starting (Alarm currently active)")
        }

        if reset_active {
            self.current_active = None;
            debug!("Reset active to None");
        }
    }
}

impl Default for LedScheduler {
    fn default() -> Self {
        let mut file =
            File::open(SCHEDULE_FILE).expect("Unable to open schedule file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read file");

        let alarms: Vec<LedAlarm> = serde_json::from_str(&contents)
            .expect("Unable to parse JSON schedule file");

        for alarm in &alarms {
            debug!(
                "Setting alarm: {:?} {} {}",
                alarm.days, alarm.hour, alarm.minute
            );
        }

        Self {
            alarms,
            current_active: None,
        }
    }
}
