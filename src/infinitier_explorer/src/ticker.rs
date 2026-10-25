//! A timer that advances the viewer while it plays something (a BAM
//! animation, a sound, a movie) and is stopped otherwise.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use slint::{ComponentHandle, Timer, TimerMode};

use crate::AppWindow;
use crate::callbacks::AppRc;
use crate::view;

#[derive(Default)]
pub struct Ticker {
    timer: Timer,
    /// The interval the timer runs at; `None` while stopped.
    interval: Cell<Option<Duration>>,
}

impl Ticker {
    /// Run, stop or re-time the timer for the viewer's current needs.
    pub fn sync(self: &Rc<Self>, ui: &AppWindow, app: &AppRc) {
        let wanted = app.borrow().viewer.tick_interval();
        if wanted == self.interval.get() {
            return;
        }
        self.interval.set(wanted);
        let Some(interval) = wanted else {
            self.timer.stop();
            return;
        };
        let ui = ui.as_weak();
        let app = app.clone();
        let ticker = Rc::downgrade(self);
        self.timer.start(TimerMode::Repeated, interval, move || {
            let (Some(ui), Some(ticker)) = (ui.upgrade(), ticker.upgrade()) else {
                return;
            };
            {
                let mut app = app.borrow_mut();
                app.viewer.tick();
                view::refresh_viewer(&ui, &app);
            }
            // The playback may have ended.
            ticker.sync(&ui, &app);
        });
    }
}
