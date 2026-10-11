use log::Level;

use crate::colors::{Color, ColorPosition};

pub struct LogLevelConfig {
    pub stderr: bool,

    pub show_module: bool,
    pub show_file: bool,
    pub show_line: bool,

    pub level_sign: Option<&'static str>,

    pub fg_color: Color,
    pub bg_color: Color,

    pub color_position: ColorPosition,
}

impl LogLevelConfig {
    pub fn default_from_level(level: Level) -> Self {
        let mut config = Self::default();
        config.color_position = ColorPosition::LevelOnly;

        match level {
            Level::Error => {
                config.level_sign = Some("E");
                config.fg_color = Color::Red;
                config.show_module = true;
                config.show_file = true;
                config.show_line = true;
                config.color_position = ColorPosition::WholeLine;

                config.stderr = true;
            }
            Level::Warn => {
                config.level_sign = Some("W");
                config.fg_color = Color::Yellow;
                config.show_module = true;
                config.show_file = true;
                config.show_line = false;
                config.color_position = ColorPosition::WholeLine;

                config.stderr = true;
            }
            Level::Info => {
                config.level_sign = Some("I");
                config.fg_color = Color::Green;
                config.show_module = false;
                config.show_file = false;
                config.show_line = false;
            }
            Level::Debug => {
                config.level_sign = Some("D");
                config.fg_color = Color::White;
                config.show_module = false;
                config.show_file = false;
                config.show_line = false;
            }
            Level::Trace => {
                config.level_sign = Some("T");
                config.fg_color = Color::BrightBlack;
                config.show_module = false;
                config.show_file = false;
                config.show_line = false;
            }
        }

        config
    }
}

impl Default for LogLevelConfig {
    fn default() -> Self {
        Self {
            stderr: false,

            show_module: true,
            show_file: false,
            show_line: false,

            level_sign: None,

            fg_color: Color::White,
            bg_color: Color::Black,

            color_position: ColorPosition::WholeLine,
        }
    }
}

pub struct LogLevelsConfig(pub [LogLevelConfig; 5]);
impl LogLevelsConfig {
    pub fn get_level(&self, level: Level) -> &LogLevelConfig {
        &self.0[level as usize - 1]
    }

    pub fn get_level_mut(&mut self, level: Level) -> &mut LogLevelConfig {
        &mut self.0[level as usize - 1]
    }
}

impl Default for LogLevelsConfig {
    fn default() -> Self {
        LogLevelsConfig([
            LogLevelConfig::default_from_level(Level::Error),
            LogLevelConfig::default_from_level(Level::Warn),
            LogLevelConfig::default_from_level(Level::Info),
            LogLevelConfig::default_from_level(Level::Debug),
            LogLevelConfig::default_from_level(Level::Trace),
        ])
    }
}
