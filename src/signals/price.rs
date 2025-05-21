use egostrategy_datahub::models::stock::DailyData as DailyBar;
use super::BuySignalGenerator;

/// 基于收盘价的买入信号生成器
pub struct ClosePriceSignal;

impl BuySignalGenerator for ClosePriceSignal {
    fn name(&self) -> String {
        String::from("次日收盘价买入")
    }
    
    fn calculate_buy_price(&self, _symbol: &str, data: &[DailyBar], forecast_idx: usize) -> f32 {
        let idx = data.len().saturating_sub(forecast_idx).saturating_sub(1);
        if idx < data.len() {
            data[idx].close
        } else {
            0.0
        }
    }
}

/// 基于开盘价的买入信号生成器
pub struct OpenPriceSignal;

impl BuySignalGenerator for OpenPriceSignal {
    fn name(&self) -> String {
        String::from("次日开盘价买入")
    }
    
    fn calculate_buy_price(&self, _symbol: &str, data: &[DailyBar], forecast_idx: usize) -> f32 {
        let idx = data.len().saturating_sub(forecast_idx).saturating_sub(1);
        if idx < data.len() {
            data[idx].open
        } else {
            0.0
        }
    }
}

/// 基于限价的买入信号生成器
pub struct LimitPriceSignal {
    pub price_ratio: f32,  // 相对于前一日收盘价的比例
}

impl Default for LimitPriceSignal {
    fn default() -> Self {
        Self {
            price_ratio: 0.98,  // 默认以前一日收盘价的98%买入
        }
    }
}

impl BuySignalGenerator for LimitPriceSignal {
    fn name(&self) -> String {
        format!("次日限价买入({}%)", self.price_ratio * 100.0)
    }
    
    fn calculate_buy_price(&self, _symbol: &str, data: &[DailyBar], forecast_idx: usize) -> f32 {
        let idx = data.len().saturating_sub(forecast_idx).saturating_sub(1);
        if idx > 0 && idx < data.len() {
            data[idx-1].close * self.price_ratio
        } else {
            0.0
        }
    }
}
