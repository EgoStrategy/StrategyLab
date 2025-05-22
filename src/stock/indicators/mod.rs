pub mod trend;
pub mod oscillator;
pub mod volatility;
pub mod utils;

// 重新导出常用函数，方便使用
pub use trend::{calculate_ema, moving_average, calculate_macd};
pub use oscillator::{calculate_rsi, calculate_stochastic, calculate_momentum};
pub use volatility::{standard_deviation, calculate_atr, calculate_bollinger_bands, calculate_keltner_channel};
pub use utils::{extract_price_data, calculate_price_change, calculate_cumulative_return, calculate_max_drawdown, calculate_sharpe_ratio};
