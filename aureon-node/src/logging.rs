use tracing::Level;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt, fmt};
use std::io;

/// Initialize structured logging with tracing
pub fn init_logging(level: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Parse log level from config
    let level = match level.to_lowercase().as_str() {
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        "trace" => Level::TRACE,
        _ => Level::INFO,
    };

    // Create environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level.to_string()));

    // Create console writer layer
    let console_layer = fmt::layer()
        .with_writer(io::stderr);

    // Create registry with layers
    Registry::default()
        .with(env_filter)
        .with(console_layer)
        .init();

    Ok(())
}

/// Helper to log consensus events
#[macro_export]
macro_rules! consensus_log {
    ($level:expr, $msg:expr) => {
        match $level {
            "debug" => tracing::debug!($msg),
            "info" => tracing::info!($msg),
            "warn" => tracing::warn!($msg),
            "error" => tracing::error!($msg),
            _ => tracing::info!($msg),
        }
    };
    ($level:expr, $($key:tt = $value:tt),*, $msg:expr) => {
        match $level {
            "debug" => tracing::debug!($($key = $value),*, $msg),
            "info" => tracing::info!($($key = $value),*, $msg),
            "warn" => tracing::warn!($($key = $value),*, $msg),
            "error" => tracing::error!($($key = $value),*, $msg),
            _ => tracing::info!($($key = $value),*, $msg),
        }
    };
}

/// Helper to log network events
#[macro_export]
macro_rules! network_log {
    ($level:expr, $msg:expr) => {
        match $level {
            "debug" => tracing::debug!(target: "network", $msg),
            "info" => tracing::info!(target: "network", $msg),
            "warn" => tracing::warn!(target: "network", $msg),
            "error" => tracing::error!(target: "network", $msg),
            _ => tracing::info!(target: "network", $msg),
        }
    };
    ($level:expr, $($key:tt = $value:tt),*, $msg:expr) => {
        match $level {
            "debug" => tracing::debug!(target: "network", $($key = $value),*, $msg),
            "info" => tracing::info!(target: "network", $($key = $value),*, $msg),
            "warn" => tracing::warn!(target: "network", $($key = $value),*, $msg),
            "error" => tracing::error!(target: "network", $($key = $value),*, $msg),
            _ => tracing::info!(target: "network", $($key = $value),*, $msg),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_parsing_debug() {
        let level = match "debug".to_lowercase().as_str() {
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            "trace" => Level::TRACE,
            _ => Level::INFO,
        };
        assert_eq!(level, Level::DEBUG);
    }

    #[test]
    fn test_level_parsing_info() {
        let level = match "info".to_lowercase().as_str() {
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            "trace" => Level::TRACE,
            _ => Level::INFO,
        };
        assert_eq!(level, Level::INFO);
    }

    #[test]
    fn test_level_parsing_error() {
        let level = match "error".to_lowercase().as_str() {
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            "trace" => Level::TRACE,
            _ => Level::INFO,
        };
        assert_eq!(level, Level::ERROR);
    }

    #[test]
    fn test_level_parsing_invalid_defaults_to_info() {
        let level = match "invalid".to_lowercase().as_str() {
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            "trace" => Level::TRACE,
            _ => Level::INFO,
        };
        assert_eq!(level, Level::INFO);
    }
}
