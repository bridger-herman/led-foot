use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use chrono::prelude::*;
use serde_derive::{Deserialize, Serialize};
use serde_json;

use crate::led_state::{set_interrupt, LED_SYSTEM, ROOM_MANAGER, WEMO_MANAGER};
use crate::room_manager::ScheduledRoomState;

const SCHEDULE_FILE: &str = "schedule.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LedAlarm {
    days: Vec<String>,
    hour: String,
    minute: String,

    /// Is the alarm enabled?
    enabled: Option<bool>,

    /// The sequence to be used (given in png format)
    sequence: Option<String>,

    /// Which rooms to enable with this alarm
    rooms: Option<ScheduledRoomState>,

    /// Which wemos commands to give with this alarm; Map of (WeMo Device, Command)
    /// Command is "on", "off", or "toggle"
    wemos: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct LedScheduler {
    pub alarms: Vec<LedAlarm>,

    /// Is any alarm currently active this minute?
    /// Used to avoid double-starting any scheduled alarms.
    current_minute_active: Option<LedAlarm>,
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
        debug!("Wrote schedule file {}", SCHEDULE_FILE);
    }

    pub fn one_frame(&mut self) {
        let now = Local::now();

        let now_weekday = &format!("{:02?}", now.weekday());
        let now_hour = &format!("{:02?}", now.hour());
        let now_minute = &format!("{:02?}", now.minute());

        let reset_active = if let Some(LedAlarm { ref minute, .. }) =
            self.current_minute_active
        {
            minute != now_minute
        } else {
            false
        };

        if self.current_minute_active.is_none() {
            for alarm in &self.alarms {
                // First, confirm that the alarm is actually enabled
                // Default to true if there's no entry for "enabled"
                if let Some(enabled) = alarm.enabled {
                    if !enabled {
                        continue;
                    }
                }

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
                        // Signal that we need to interrupt the current sequence
                        set_interrupt(true);

                        // See if we need to turn on/off any rooms...
                        if let Some(ref rooms) = alarm.rooms {
                            if let Ok(mut man) = ROOM_MANAGER.get().write() {
                                man.set_active_rooms_option(rooms);
                            }
                        }

                        // See if we need to turn on/off any wemos...
                        if let Some(ref wemos) = alarm.wemos {
                            for (wemo, cmd) in wemos.iter() {
                                WEMO_MANAGER.get().send_wemo_command(wemo, cmd);
                            }
                        }

                        // Then, spawn a thread to handle the actual LED code
                        if let Some(ref seq) = alarm.sequence {
                            let alarm_copy_sequence = seq.clone();
                            std::thread::spawn(move || {
                                if let Ok(mut sys) = LED_SYSTEM.get().write() {
                                    sys.update_sequence(&alarm_copy_sequence);
                                    sys.run_sequence();
                                } else {
                                    error!(
                                        "Unable to acquire lock on LED system"
                                    );
                                };
                            });
                        }

                        self.current_minute_active = Some(alarm.clone());
                    }
                }
            }
        } else {
            trace!("Not starting (Alarm currently active)")
        }

        if reset_active {
            self.current_minute_active = None;
            debug!("Reset active to None");
        }
    }

    pub fn try_from_schedule_file() -> Result<Self, std::io::Error> {
        let mut file = File::open(SCHEDULE_FILE)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let alarms: Vec<LedAlarm> = serde_json::from_str(&contents)
            .expect("Unable to parse JSON schedule file");

        for alarm in &alarms {
            debug!("Setting alarm: {:#?}", alarm);
        }

        Ok(Self {
            alarms,
            current_minute_active: None,
        })
    }
}

impl Default for LedScheduler {
    fn default() -> Self {
        if let Ok(schedule) = Self::try_from_schedule_file() {
            schedule
        } else {
            warn!("No schedule was detected or schedule was corrupt, initializing blank schedule");
            Self {
                alarms: vec![],
                current_minute_active: None,
            }
        }
    }
}
