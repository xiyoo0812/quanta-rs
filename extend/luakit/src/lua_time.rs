#![allow(non_snake_case)]
#![allow(dead_code)]

use std::thread;
use once_cell::sync::OnceCell;
use std::time::{ Duration, Instant, SystemTime, UNIX_EPOCH};

static STEADY_EPOCH: OnceCell<Instant> = OnceCell::new();

fn get_steady_epoch() -> &'static Instant {
    STEADY_EPOCH.get_or_init(Instant::now)
}

pub fn now() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(_) => 0
    }
}

pub fn now_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0
    }
}

pub fn now_ns() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos() as u64,
        Err(_) => 0
    }
}

pub fn steady() -> u64 {
     Instant::now().duration_since(*get_steady_epoch()).as_secs()
}

pub fn steady_ms() -> u64 {
    Instant::now().duration_since(*get_steady_epoch()).as_millis() as u64
}

pub fn sleep_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}