use crate::signals::BuySignalGenerator;
use egostrategy_datahub::models::stock::DailyData as DailyBar;
use log::debug;

/// 地包天买入信号生成器
pub struct BottomReverseSignal {
    pub price_buffer_percent: f32,  // 买入价格缓冲比例
}

impl Default for BottomReverseSignal {
    fn default() -> Self {
        Self {
            price_buffer_percent: 1.0,  // 默认买入价格为开盘价上浮1%
        }
    }
}

impl BuySignalGenerator for BottomReverseSignal {
    fn name(&self) -> String {
        format!("地包天买入信号")
    }
    
    fn calculate_buy_price(&self, symbol: &str, data: &[DailyBar], forecast_idx: usize) -> f32 {
        if data.len() <= forecast_idx + 1 {
            return 0.0;
        }
        
        // 检查是否形成地包天形态（低开高走）
        let today = &data[forecast_idx];
        let yesterday = &data[forecast_idx + 1];
        
        // 地包天形态：今日开盘低于昨日最低价，收盘高于昨日最高价
        let is_bottom_reverse = today.open < yesterday.low && today.close > yesterday.high;
        
        // 辅助条件：今日成交量小于昨日（地量企稳）
        let is_volume_low = (today.volume as f32) < (yesterday.volume as f32 * 0.9);
        
        // 计算买入价格（次日开盘价上浮一定比例）
        let buy_price = if forecast_idx > 0 {
            data[forecast_idx - 1].open * (1.0 + self.price_buffer_percent / 100.0)
        } else {
            today.close * (1.0 + self.price_buffer_percent / 100.0)
        };
        
        // 判断是否生成买入信号
        if is_bottom_reverse || (today.close > today.open && is_volume_low) {
            debug!("生成买入信号: {}, 买入价={:.2}", symbol, buy_price);
            return buy_price;
        }
        
        0.0  // 不满足条件，返回0表示不买入
    }
}
