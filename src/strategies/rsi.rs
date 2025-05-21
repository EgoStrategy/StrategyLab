use egostrategy_datahub::models::stock::DailyData as DailyBar;
use crate::strategies::StockSelector;

/// RSI选股策略
pub struct RsiSelector {
    top_n: usize,
    period: usize,
    oversold_threshold: f32,
}

impl RsiSelector {
    /// 创建新的RSI选股策略
    pub fn new(top_n: usize, period: usize, oversold_threshold: f32) -> Self {
        Self { 
            top_n,
            period,
            oversold_threshold,
        }
    }
    
    /// 计算RSI值
    fn calculate_rsi(&self, data: &[DailyBar], idx: usize) -> f32 {
        if idx < self.period || data.len() <= idx {
            return 50.0; // 默认中性值
        }
        
        let mut gain_sum = 0.0;
        let mut loss_sum = 0.0;
        
        // 计算过去period天的价格变化
        for i in (idx - self.period + 1)..=idx {
            let price_change = data[i].close - data[i-1].close;
            if price_change >= 0.0 {
                gain_sum += price_change;
            } else {
                loss_sum += price_change.abs();
            }
        }
        
        // 避免除以零
        if loss_sum == 0.0 {
            return 100.0;
        }
        
        let rs = gain_sum / loss_sum;
        let rsi = 100.0 - (100.0 / (1.0 + rs));
        
        rsi
    }
}

impl StockSelector for RsiSelector {
    fn name(&self) -> String {
        format!("RSI({})选股策略", self.period)
    }
    
    fn top_n(&self) -> usize {
        self.top_n
    }
    
    fn calculate_score(&self, _symbol: &str, data: &[DailyBar], forecast_idx: usize) -> f32 {
        if data.len() <= forecast_idx || forecast_idx < self.period {
            return 0.0;
        }
        
        let rsi = self.calculate_rsi(data, forecast_idx);
        let rsi_prev = self.calculate_rsi(data, forecast_idx - 1);
        
        // RSI超卖区域反转时，得分较高
        if rsi_prev < self.oversold_threshold && rsi > rsi_prev {
            return 100.0 - rsi + (rsi - rsi_prev) * 5.0;
        } else if rsi < self.oversold_threshold {
            return 50.0 - rsi;
        } else {
            return 0.0;
        }
    }
}
