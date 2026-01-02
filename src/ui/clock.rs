use std::{cmp, sync};

#[derive(Default)]
pub(super) struct ClockInner {
    tps: Option<u64>,

    last_now: u64,
    last_system_time: u64,
    last_updated: u64,

    actual_fps: f64,
    actual_tps: f64,
    prev_tps: u64,
    fps_count: u64,
    tps_count: u64,
}

#[derive(Default)]
pub struct Clock(sync::Mutex<ClockInner>);

const DEFAULT_TPS: u64 = 60;

impl ClockInner {
    pub fn new() -> Self {
        let now = chrono::Local::now().timestamp_millis() as u64;

        Self {
            tps: Some(DEFAULT_TPS),

            last_now: now,
            last_system_time: now,
            last_updated: now,

            actual_fps: 0.,
            actual_tps: 0.,
            prev_tps: 0,
            fps_count: 0,
            tps_count: 0,
        }
    }

    pub(super) fn calc_count_from_tps(&mut self, tps: u64, now: u64) -> u64 {
        if tps == 0 {
            return 0;
        }

        if now < self.last_system_time {
            return 0;
        }
        let diff = now - self.last_system_time;
        let (sync_with_system_clock, mut count) =
            if diff > cmp::max(5000 / tps, 5000 / 60) || self.prev_tps != tps {
                (true, 0)
            } else {
                (false, diff * tps / 1000)
            };
        self.prev_tps = tps;

        if count == 0 && 1000 / tps / 2 < diff {
            count = 1;
        }
        if count == 2 && 1000 / tps * 3 / 2 > diff {
            count = 1;
        }

        self.last_system_time = if sync_with_system_clock {
            now
        } else {
            count * 1000 / tps
        };

        count
    }

    fn update_fps_and_tps(&mut self, now: u64, count: u64) {
        self.fps_count += 1;
        self.tps_count += count;

        if 1000 > now - self.last_updated {
            return;
        }

        self.actual_fps = self.fps_count as f64 * 1000. / (now - self.last_updated) as f64;
        self.actual_tps = self.tps_count as f64 * 1000. / (now - self.last_updated) as f64;
        self.last_updated = now;
        self.fps_count = 0;
        self.tps_count = 0;
    }

    pub(crate) fn update_frame(&mut self) -> (u64, f64) {
        let now = chrono::Local::now().timestamp_millis() as u64;
        self.last_now = now;

        let count = if let Some(tps) = self.tps {
            self.calc_count_from_tps(tps, now)
        } else {
            1
        };
        self.update_fps_and_tps(now, count);

        (count, self.actual_fps)
    }
}

impl Clock {
    pub fn new() -> Self {
        Self(sync::Mutex::new(ClockInner::new()))
    }

    pub fn update_frame(&self) -> (u64, f64) {
        let Ok(mut clock) = self.0.lock() else {
            return (0, 0.);
        };
        clock.update_frame()
    }

    pub fn set_tps(&self, tps: Option<u64>) {
        let Ok(mut clock) = self.0.lock() else {
            return;
        };
        clock.tps = tps;
    }

    pub fn get_tps(&self) -> Option<u64> {
        let Ok(clock) = self.0.lock() else {
            return None;
        };
        clock.tps
    }
}
