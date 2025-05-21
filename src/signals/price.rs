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
