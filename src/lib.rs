pub mod backtest;
pub mod signals;
pub mod stock;
pub mod strategies;
pub mod targets;
pub mod scorecard;
pub mod utils;

// Re-export commonly used types
pub use backtest::{BacktestEngine, BacktestResult};
pub use signals::BuySignalGenerator;
pub use strategies::StockSelector;
pub use targets::Target;
pub use scorecard::Scorecard;
