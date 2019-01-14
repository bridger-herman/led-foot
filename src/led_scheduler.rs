use std::fs::File;
use std::io::prelude::*;

use schedule::{Agenda, Job};

const SCHEDULE_FILE: &str = "schedules/schedule.txt";

pub struct LedScheduler {
    agenda: Agenda<'static>,
}

impl LedScheduler {
    pub fn one_frame(&mut self) {
        self.agenda.run_pending();
    }
}

impl Default for LedScheduler {
    fn default() -> Self {
        let mut file =
            File::open(SCHEDULE_FILE).expect("Unable to open schedule file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read file");

        let lines_parts: Vec<Vec<String>> = contents
            .split('\n')
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .map(|line| line.split(';').map(|part| part.to_string()).collect())
            .collect();

        let jobs: Vec<_> = lines_parts
            .into_iter()
            .map(|parts| {
                assert_eq!(parts.len(), 2);
                let (cron, name) = (parts[0].clone(), parts[1].clone());
                println!("Setting {} on schedule for {}", name, cron);
                Job::new(
                    move || {
                        println!("Changing to {} on schedule", name);
                        led_system!()
                            .update_sequence(&format!("./sequences/{}", name));
                        led_system!().run_sequence();
                    },
                    cron.parse().unwrap(),
                )
            })
            .collect();

        let mut agenda = Agenda::new();
        for job in jobs {
            agenda.add(job);
        }
        Self { agenda }
    }
}
