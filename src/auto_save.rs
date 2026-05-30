pub struct AutoSaveState {
    pub enabled: bool,
    pub interval_secs: f64,
    pub last_edit_time: f64,
    pub pending_save: bool,
}

impl AutoSaveState {
    pub fn new() -> Self {
        Self {
            enabled: true,
            interval_secs: 60.0,
            last_edit_time: 0.0,
            pending_save: false,
        }
    }

    pub fn on_edit(&mut self, current_time: f64) {
        self.last_edit_time = current_time;
        self.pending_save = true;
    }

    pub fn check(&mut self, current_time: f64) -> bool {
        if !self.enabled || !self.pending_save { return false; }
        let elapsed = current_time - self.last_edit_time;
        if elapsed >= self.interval_secs {
            self.pending_save = false;
            return true;
        }
        false
    }
}
