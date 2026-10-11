use log::{Level, LevelFilter, Log, SetLoggerError};

use crate::{
    colors::{Color, ColorPosition},
    config::{LogLevelConfig, LogLevelsConfig},
};

pub struct Logger {
    max_level: LevelFilter,
    level_config: LogLevelsConfig,
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.max_level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level_config: &LogLevelConfig = self.level_config.get_level(record.level());

        let print = |text: &str| self.print_correct_std(&level_config, text);

        if matches!(level_config.color_position, ColorPosition::WholeLine) {
            print(level_config.fg_color.get_fg_color_ansi_text());
            print(level_config.bg_color.get_bg_color_ansi_text());
        }

        print("[");
        if matches!(level_config.color_position, ColorPosition::LevelOnly)
            && let Some(sign) = level_config.level_sign
        {
            print(level_config.fg_color.get_fg_color_ansi_text());
            print(level_config.bg_color.get_bg_color_ansi_text());
            print(sign);
            print(Color::RESET_TEXT_COLOR);
        } else if let Some(sign) = level_config.level_sign {
            print(sign);
        }
        print("] ");

        if level_config.show_module
            && let Some(module) = record.module_path()
        {
            print("[");
            print(module);
            print("] ");
        }
        if level_config.show_file
            && let Some(file) = record.file()
        {
            print("[");
            print(file);
            print("] ");
        }
        if level_config.show_line
            && let Some(line) = record.line()
        {
            print("[");
            print(&line.to_string());
            print("] ");
        }

        print(&record.args().to_string());

        print(Color::RESET_TEXT_COLOR);

        print("\n");
    }

    fn flush(&self) {}
}

impl Logger {
    fn print_correct_std(&self, config: &LogLevelConfig, text: &str) {
        if !config.stderr {
            print!("{}", text);
        } else {
            eprint!("{}", text);
        }
    }
}

pub struct LoggerBuilder {
    max_level: LevelFilter,
    level_config: LogLevelsConfig,
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        LoggerBuilder {
            max_level: LevelFilter::Trace,
            level_config: LogLevelsConfig::default(),
        }
    }
}

impl LoggerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_max_level(&mut self, level: LevelFilter) {
        self.max_level = level;
    }

    pub fn set_level_config(&mut self, level: Level, config: LogLevelConfig) {
        *self.level_config.get_level_mut(level) = config;
    }

    pub fn build(self) -> Result<(), SetLoggerError> {
        let logger = Box::new(self.get_logger());
        let max_level = logger.max_level;
        log::set_boxed_logger(logger).map(|()| log::set_max_level(max_level))
    }

    pub fn build_default() -> Result<(), SetLoggerError> {
        Self::default().build()
    }

    pub fn get_logger(self) -> Logger {
        Logger {
            max_level: self.max_level,
            level_config: self.level_config,
        }
    }

    pub fn get_default_logger() -> Logger {
        Self::default().get_logger()
    }
}
