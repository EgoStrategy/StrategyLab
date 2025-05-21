use egostrategy_datahub::models::stock::DailyData as DailyBar;
use super::BuySignalGenerator;

/// 基于收盘价的买入信号生成器 - 适用于倒序数据
pub struct ClosePriceSignal;

impl BuySignalGenerator for ClosePriceSignal {
    fn name(&self) -> String {
        String::from("次日收盘价买入")
    }
    
    fn calculate_buy_price(&self, symbol: &str, data: &[DailyBar], forecast_idx: usize) -> f32 {
        // 对于倒序数据，forecast_idx表示从最新数据往后数的天数
        // 次日收盘价就是forecast_idx+1的收盘价
        if data.len() <= forecast_idx + 1 {
            log::debug!("股票 {}: 计算收盘价买入信号失败, forecast_idx={}, len={}", 
                symbol, forecast_idx, data.len());
            return 0.0;
        }
        
        let price = data[forecast_idx + 1].close;
        log::debug!("股票 {}: 计算收盘价买入信号, forecast_idx={}, price={:.2}", 
            symbol, forecast_idx, price);
        price
    }
}

/// 基于开盘价的买入信号生成器 - 适用于倒序数据
pub struct OpenPriceSignal;

impl BuySignalGenerator for OpenPriceSignal {
    fn name(&self) -> String {
        String::from("次日开盘价买入")
    }
    
    fn calculate_buy_price(&self, symbol: &str, data: &[DailyBar], forecast_idx: usize) -> f32 {
        // 对于倒序数据，forecast_idx表示从最新数据往后数的天数
        // 次日开盘价就是forecast_idx+1的开盘价
        if data.len() <= forecast_idx + 1 {
            log::debug!("股票 {}: 计算开盘价买入信号失败, forecast_idx={}, len={}", 
                symbol, forecast_idx, data.len());
            return 0.0;
        }
        
        let price = data[forecast_idx + 1].open;
        log::debug!("股票 {}: 计算开盘价买入信号, forecast_idx={}, price={:.2}", 
            symbol, forecast_idx, price);
        price
    }
}

/// 基于限价的买入信号生成器 - 适用于倒序数据
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
    
    fn calculate_buy_price(&self, symbol: &str, data: &[DailyBar], forecast_idx: usize) -> f32 {
        // 对于倒序数据，forecast_idx表示从最新数据往后数的天数
        // 前一日收盘价就是forecast_idx的收盘价
        if data.len() <= forecast_idx {
            log::debug!("股票 {}: 计算限价买入信号失败, forecast_idx={}, len={}", 
                symbol, forecast_idx, data.len());
            return 0.0;
        }
        
        let price = data[forecast_idx].close * self.price_ratio;
        log::debug!("股票 {}: 计算限价买入信号, forecast_idx={}, 前收={:.2}, 比例={}%, 价格={:.2}", 
            symbol, forecast_idx, data[forecast_idx].close, self.price_ratio * 100.0, price);
        price
    }
}
