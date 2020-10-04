extern crate chrono;
extern crate clap;
extern crate timer;

use std::convert::TryFrom;
use std::io;
use std::io::BufRead;
use std::process::exit;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Duration, Local};
use clap::{App, Arg};

fn main() {
    let args = App::new("stale")
        .version("0.1.0")
        .author("christheblog")
        .about("Detect when stdout is stale")
        .arg(
            Arg::with_name("delay")
                .long("delay")
                .short("d")
                .help("Sets the delay after which stdout stream is considered stale.")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("passthrough")
                .long("passthrough")
                .short("p")
                .help("Prints the original stream back in stdout")
                .required(false)
                .takes_value(false),
        )
        .arg(
            Arg::with_name("message")
                .long("message")
                .short("m")
                .help("Customize stale message")
                .required(false)
                .takes_value(true)
                .default_value("[{now}] stream is stale since {staletime}"),
        )
        .arg(
            Arg::with_name("exit")
                .long("exit")
                .short("e")
                .help("Exit process with provided exit code when stale stream is detected")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("no-rearm")
                .long("no-rearm")
                .short("n")
                .help("Detection will not be reamed once it has fired.")
                .required(false)
                .takes_value(false),
        )
        .get_matches();

    // Reading arguments
    let delay = args
        .value_of("delay")
        .map(|x| {
            x.parse::<u64>()
                .expect("Delay should be a positive integer")
        })
        .map(|x| Duration::seconds(x as i64))
        .unwrap();
    let exit_code = args
        .value_of("exit")
        .map(|x| x.parse::<i32>().expect("Exit code should be an integer"));
    let alert_message = args.value_of("message").unwrap();
    let passthrough = args.is_present("passthrough");
    let no_rearm = args.is_present("no-rearm");

    // Timestamp of the last line received from Stdin
    let last_seen = Arc::new(RwLock::new(timestamp_ms()));
    // Timer - detecting stale
    let timer = timer::Timer::new();
    let mut trigger_armed = true;
    let last_seen_for_timer = last_seen.clone();
    let alert = alert_message.to_string();
    let delay_converted = u128::try_from(delay.num_milliseconds())
        .expect("Duration couldn't be converted to u128 timestamp");
    // Scheduling a repeated task
    let guard = timer.schedule_repeating(delay, move || {
        if let Ok(seen) = last_seen_for_timer.read() {
            let now = timestamp_ms();
            if is_stale(now, *seen, delay_converted) && trigger_armed {
                println!("{}", substitute_datetime(&alert, now, *seen));
                // Disarming the trigger. No alert will be ever triggered
                if no_rearm {
                    trigger_armed = false;
                }
                // If exit_code is provided, we exit the process after printing the alert
                match exit_code {
                    Some(code) => exit(code),
                    None => (),
                }
            }
        }
    });
    // Reading Stdin
    let stdin = io::stdin();
    for line_result in stdin.lock().lines() {
        let now = timestamp_ms();
        // Updating "last seen" timestamp. This will be read by the scheduled task
        if let Ok(mut seen) = last_seen.clone().write() {
            *seen = now;
        }
        // Passthrough means we reforward read line to stdout
        if passthrough {
            forward_output(line_result);
        }
    }

    // We need an explicit drop here to keep the guard in scope
    // else the scheduled task will be cancelled straight away
    drop(guard);
}

fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error when getting timestamp")
        .as_millis()
}

// Conversion between u128 timestamp and chrono::DateTime
fn to_datetime(ts: u128) -> DateTime<Local> {
    let ts_millis = UNIX_EPOCH + std::time::Duration::from_millis(ts as u64);
    DateTime::<Local>::from(ts_millis)
}

fn substitute_datetime(msg: &str, now: u128, last_seen: u128) -> String {
    msg.replace("{staletime}", &format!("{}", to_datetime(last_seen)))
       .replace("{now}", &format!("{}", to_datetime(now)))
}

fn is_stale(now: u128, last_seen: u128, delay: u128) -> bool {
    now - delay >= last_seen
}

fn forward_output(line: Result<String, std::io::Error>) -> () {
    match line {
        Ok(line) => println!("{}", line),
        Err(_) => (),
    }
}
