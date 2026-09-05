//! 开箱即用的 tracing 日志配置。

use crate::error::{Result, SdkError};
use std::{io::Write, path::PathBuf};
use time::macros::format_description;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::{EnvFilter, fmt};

/// 日志输出格式。
#[derive(Debug, Clone, Copy)]
pub enum Format {
    Pretty,
    Json,
}

/// 日志输出目标。
#[derive(Debug, Clone)]
pub enum LogTarget {
    /// 标准输出。
    Stdout,
    /// 标准错误。
    Stderr,
    /// 按天滚动的文件。
    FileDaily(PathBuf),
    /// 按小时滚动的文件。
    FileHourly(PathBuf),
}

/// 日志初始化守卫；文件日志必须保持该值存活以完成异步写入。
pub type WorkerGuard = tracing_appender::non_blocking::WorkerGuard;

/// Sakura 风格的日志 Builder（名称沿用原项目提示词）。
#[derive(Debug, Clone)]
pub struct SakuraLogger {
    format: Format,
    ansi: bool,
    level: String,
    target: LogTarget,
}

impl Default for SakuraLogger {
    fn default() -> Self {
        Self {
            format: Format::Pretty,
            ansi: true,
            level: "info".into(),
            target: LogTarget::Stdout,
        }
    }
}

impl SakuraLogger {
    /// 返回默认漂亮格式 Builder。
    pub fn builder() -> Self {
        Self::default()
    }

    /// 使用默认配置初始化全局日志订阅器。
    pub fn init() -> Result<WorkerGuard> {
        Self::default().try_init()
    }

    /// 切换输出格式。
    pub fn with_format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }
    /// 开关 ANSI 彩色输出。
    pub fn with_ansi(mut self, ansi: bool) -> Self {
        self.ansi = ansi;
        self
    }
    /// 设置默认日志级别；若设置了 `RUST_LOG`，环境变量优先。
    pub fn with_level(mut self, level: impl Into<String>) -> Self {
        self.level = level.into();
        self
    }
    /// 设置输出目标。
    pub fn with_target(mut self, target: LogTarget) -> Self {
        self.target = target;
        self
    }

    /// 初始化全局订阅器并返回异步写入守卫。
    pub fn try_init(self) -> Result<WorkerGuard> {
        let writer: Box<dyn Write + Send> = match &self.target {
            LogTarget::Stdout => Box::new(std::io::stdout()),
            LogTarget::Stderr => Box::new(std::io::stderr()),
            LogTarget::FileDaily(path) => Box::new(rolling::daily(path, "qqbot.log")),
            LogTarget::FileHourly(path) => Box::new(rolling::hourly(path, "qqbot.log")),
        };
        let (writer, guard) = non_blocking(writer);
        let filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(self.level));
        let result = match self.format {
            Format::Pretty => fmt::Subscriber::builder()
                .with_env_filter(filter)
                .with_writer(writer)
                .with_ansi(self.ansi)
                .with_timer(UtcTime::new(format_description!(
                    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
                )))
                .pretty()
                .try_init(),
            Format::Json => fmt::Subscriber::builder()
                .with_env_filter(filter)
                .with_writer(writer)
                .with_ansi(false)
                .with_timer(UtcTime::new(format_description!(
                    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
                )))
                .json()
                .with_current_span(true)
                .with_span_list(true)
                .try_init(),
        };
        result.map_err(|e| SdkError::Logging(e.to_string()))?;
        Ok(guard)
    }
}
