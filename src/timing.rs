//! A refresh-rate-based frame deadline, independent of Windows input messages.
use crate::win32::*;
use std::time::{Duration, Instant};
pub struct FramePacer {
    timer: Handle,
    period: Duration,
    next_frame: Instant,
}
impl FramePacer {
    pub fn new(hwnd: Hwnd) -> Self {
        unsafe {
            let dc = GetDC(hwnd);
            let hz = GetDeviceCaps(dc, 116);
            ReleaseDC(hwnd, dc);
            let hz = if (30..=360).contains(&hz) {
                hz as f64
            } else {
                60.0
            };
            let mut timer = CreateWaitableTimerExW(NULL, std::ptr::null(), 2, 0x1f0003);
            if timer.is_null() {
                timer = CreateWaitableTimerExW(NULL, std::ptr::null(), 0, 0x1f0003);
            }
            // Align once with composition. Waiting on DwmFlush on every frame can
            // skip refresh intervals for GDI windows, particularly in remote sessions.
            GdiFlush();
            DwmFlush();
            let period = Duration::from_secs_f64(1.0 / hz);
            Self {
                timer,
                period,
                next_frame: Instant::now() + period,
            }
        }
    }
    pub fn wait(&mut self) {
        unsafe {
            GdiFlush();
        }
        let remaining = self.next_frame.saturating_duration_since(Instant::now());
        if !remaining.is_zero() {
            unsafe {
                let due = -((remaining.as_nanos() / 100).max(1) as i64);
                if !self.timer.is_null()
                    && SetWaitableTimer(self.timer, &due, 0, NULL, NULL, 0) != 0
                {
                    WaitForSingleObject(self.timer, u32::MAX);
                } else {
                    std::thread::sleep(remaining);
                }
            }
        }
        // Advance absolute deadlines rather than accumulating rendering and wait
        // costs. Skip missed deadlines instead of issuing a burst of catch-up frames.
        self.next_frame += self.period;
        let now = Instant::now();
        while self.next_frame <= now {
            self.next_frame += self.period;
        }
    }
}
impl Drop for FramePacer {
    fn drop(&mut self) {
        if !self.timer.is_null() {
            unsafe {
                CloseHandle(self.timer);
            }
        }
    }
}
