//! User configuration.

use crate::ui::style::Theme;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub theme: Theme,
    /// Duration in milliseconds between tick events.
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(250),
            theme: Default::default(),
        }
    }
}
