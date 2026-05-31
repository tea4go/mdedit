/// 自动保存状态管理器
///
/// 监控编辑时间，在用户停止编辑超过指定间隔后触发保存。
/// 默认间隔 60 秒，只在文档有未保存修改时生效。
pub struct AutoSaveState {
    /// 是否启用自动保存
    pub enabled: bool,
    /// 自动保存间隔（秒）
    pub interval_secs: f64,
    /// 上次编辑的时间戳
    pub last_edit_time: f64,
    /// 是否有待保存的修改
    pub pending_save: bool,
}

impl AutoSaveState {
    /// 创建默认的自动保存状态（启用，60秒间隔）
    pub fn new() -> Self {
        Self {
            enabled: true,
            interval_secs: 60.0,
            last_edit_time: 0.0,
            pending_save: false,
        }
    }

    /// 编辑事件回调，记录编辑时间并标记待保存
    pub fn on_edit(&mut self, current_time: f64) {
        self.last_edit_time = current_time;
        self.pending_save = true;
    }

    /// 检查是否需要自动保存
    /// 返回 true 表示已到保存时机，同时清除待保存标志
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
