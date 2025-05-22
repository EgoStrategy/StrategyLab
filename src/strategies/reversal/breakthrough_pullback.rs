use crate::strategies::StockSelector;
use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 突破回踩选股策略
#[derive(Debug, Clone)]
pub struct BreakthroughPullbackSelector {
    pub top_n: usize,
    pub lookback_days: usize,
    pub min_breakthrough_percent: f32,
    pub max_pullback_percent: f32,
    pub volume_decline_ratio: f32,
}

impl StockSelector for BreakthroughPullbackSelector {
    fn name(&self) -> String {
        "突破回踩策略".to_string()
    }
    
    fn run(&self, stock_data: &[(String, Vec<DailyBar>)], forecast_idx: usize) -> Vec<(String, Vec<DailyBar>)> {
        let mut candidates = Vec::new();
        
        for (symbol, data) in stock_data {
            if data.len() <= forecast_idx + self.lookback_days {
                continue;
            }
            
            // 检查是否有突破
            if let Some(breakthrough_idx) = self.find_breakthrough(data, forecast_idx) {
                // 检查突破后是否有回踩
                if self.check_pullback(data, forecast_idx, breakthrough_idx) {
                    candidates.push((symbol.clone(), data.clone()));
                }
            }
        }
        
        // 如果候选股票数量超过top_n，则只返回top_n个
        if candidates.len() > self.top_n {
            candidates.truncate(self.top_n);
        }
        
        candidates
    }
}

impl BreakthroughPullbackSelector {
    /// 寻找突破点
    fn find_breakthrough(&self, data: &[DailyBar], forecast_idx: usize) -> Option<usize> {
        for i in 1..self.lookback_days {
            if forecast_idx + i >= data.len() {
                break;
            }
            
            let current = &data[forecast_idx + i];
            let prev = &data[forecast_idx + i + 1];
            
            // 检查是否有显著突破 (当日收盘价比前一日高出一定百分比)
            let breakthrough_pct = (current.close - prev.close) / prev.close * 100.0;
            
            if breakthrough_pct >= self.min_breakthrough_percent {
                // 检查成交量是否放大
                if current.volume > prev.volume {
                    return Some(forecast_idx + i);
                }
            }
        }
        
        None
    }
    
    /// 检查突破后是否有回踩
    fn check_pullback(&self, data: &[DailyBar], forecast_idx: usize, breakthrough_idx: usize) -> bool {
        if breakthrough_idx <= forecast_idx {
            return false;
        }
        
        let breakthrough_price = data[breakthrough_idx].close;
        let current_price = data[forecast_idx].close;
        
        // 计算回踩幅度
        let pullback_pct = (breakthrough_price - current_price) / breakthrough_price * 100.0;
        
        // 检查回踩幅度是否在允许范围内
        if pullback_pct > 0.0 && pullback_pct <= self.max_pullback_percent {
            // 检查回踩过程中成交量是否萎缩
            let breakthrough_volume = data[breakthrough_idx].volume as f32;
            let current_volume = data[forecast_idx].volume as f32;
            
            if current_volume <= breakthrough_volume * self.volume_decline_ratio {
                return true;
            }
        }
        
        false
    }
}
