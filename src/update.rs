use crate::settings::{get_settings_json, write_settings_json};

use chrono::{Utc, DateTime, Duration};

use std::process::Command;
use json::JsonValue;


pub fn upgrade(name: String, force: bool) {
    let mut settings_json = get_settings_json();
    if name == "" {
        let entries = settings_json.entries_mut();
        entries.for_each(|(name, value)| {
            run_command(value, force, name);
        });
    } else {
        run_command(&mut settings_json[name.clone()], force, name.as_str());
    }
    write_settings_json(json::stringify(settings_json));
}


fn run_command(value: &mut JsonValue, force: bool, name: &str) {
    let time = DateTime::parse_from_rfc3339(value["lastUpdated"].as_str().unwrap()).unwrap();
    let today_timestamp = JsonValue::from(Utc::now().to_rfc3339());
    let freq_str = value["frequency"].clone().to_string();
    let frequency = freq_str.chars();
    let time_char = frequency.clone().last().unwrap();
    let digit = frequency.clone()
        .take_while(|c| c.is_digit(10))
        .collect::<String>()
        .parse::<i64>().unwrap();
    let duration = match time_char {
        'd' => Duration::days(digit),
        'w' => Duration::weeks(digit),
        'm' => Duration::weeks(digit * 4),
        'y' => Duration::weeks(digit * 52),
        e => panic!("Invalid Character: '{}' format should be <INT><d, w, m, y>", e)
    };
    if force {
        if run_commands(value, name) {
            value["lastUpdated"] = today_timestamp.clone();
        }
    } else if time + duration < DateTime::from(Utc::now()) {
        if run_commands(value, name) {
            value["lastUpdated"] = today_timestamp.clone();
        }
    }
}

fn run_commands(value: &JsonValue, name: &str) -> bool {
    println!("Now running {}", name);
    let mut worked = true;
    value["commands"].members().for_each(|item| {
        let cmd_vec: Vec<&str> = item.as_str().unwrap()
            .split_whitespace().collect();
        match Command::new(cmd_vec[0])
            .args(&cmd_vec[1..])
            .spawn() {
            Ok(mut child) => {
                child.wait().unwrap();
            }
            Err(_) => {
                worked = false;
                eprintln!("This command does not exist.")
            }
        }
    });
    return worked;
}

