use crate::strategies::StockSelector;
use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 成交量萎缩选股策略
#[derive(Debug, Clone)]
pub struct VolumeDecliningSelector {
    pub top_n: usize,
    pub lookback_days: usize,
    pub min_consecutive_decline_days: usize,
    pub min_volume_decline_ratio: f32,
    pub price_period: usize,
    pub check_support_level: bool,
    pub max_support_ratio: f32,
}

impl Default for VolumeDecliningSelector {
    fn default() -> Self {
        Self {
            top_n: 10,
            lookback_days: 30,
            min_consecutive_decline_days: 3,
            min_volume_decline_ratio: 0.1,
            price_period: 20,
            check_support_level: true,
            max_support_ratio: 0.05,
        }
    }
}

impl StockSelector for VolumeDecliningSelector {
    fn name(&self) -> String {
        "成交量萎缩策略".to_string()
    }
    
    fn run(&self, stock_data: &[(String, Vec<DailyBar>)], forecast_idx: usize) -> Vec<(String, Vec<DailyBar>)> {
        let mut candidates = Vec::new();
        
        for (symbol, data) in stock_data {
            if data.len() <= forecast_idx + self.lookback_days {
                continue;
            }
            
            // 检查连续成交量萎缩
            if self.check_volume_decline(data, forecast_idx) {
                // 检查价格支撑位
                if !self.check_support_level || self.check_price_support(data, forecast_idx) {
                    // 计算当前价格与压力位的距离比例
                    let resistance_ratio = self.calculate_resistance_ratio(data, forecast_idx);
                    candidates.push((symbol.clone(), data.clone(), resistance_ratio));
                }
            }
        }
        
        // 根据价格距离压力位的远近排序（距离越远越好）
        candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        
        // 如果候选股票数量超过top_n，则只返回top_n个
        if candidates.len() > self.top_n {
            candidates.truncate(self.top_n);
        }
        
        // 移除排序用的比例值
        candidates.into_iter()
            .map(|(symbol, data, _)| (symbol, data))
            .collect()
    }
}

impl VolumeDecliningSelector {
    /// 检查连续成交量萎缩
    fn check_volume_decline(&self, data: &[DailyBar], forecast_idx: usize) -> bool {
        let mut consecutive_decline = 0;
        
        for i in 0..self.lookback_days - 1 {
            if forecast_idx + i + 1 >= data.len() {
                break;
            }
            
            let current_volume = data[forecast_idx + i].volume as f32;
            let prev_volume = data[forecast_idx + i + 1].volume as f32;
            
            if prev_volume > 0.0 && current_volume / prev_volume <= (1.0 - self.min_volume_decline_ratio) {
                consecutive_decline += 1;
                
                if consecutive_decline >= self.min_consecutive_decline_days {
                    return true;
                }
            } else {
                break;
            }
        }
        
        false
    }
    
    /// 检查价格支撑位
    fn check_price_support(&self, data: &[DailyBar], forecast_idx: usize) -> bool {
        if data.len() <= forecast_idx + self.price_period {
            return false;
        }
        
        // 计算支撑位 (使用最近N天的最低价)
        let mut min_price = f32::MAX;
        for i in 0..self.price_period {
            if forecast_idx + i >= data.len() {
                break;
            }
            
            min_price = min_price.min(data[forecast_idx + i].low);
        }
        
        // 检查当前价格是否接近支撑位
        let current_price = data[forecast_idx].close;
        let price_ratio = (current_price - min_price) / current_price;
        
        // 如果当前价格与支撑位相差不超过设定比例，则认为在支撑位附近
        price_ratio <= self.max_support_ratio
    }
    
    /// 计算当前价格与压力位的距离比例
    fn calculate_resistance_ratio(&self, data: &[DailyBar], forecast_idx: usize) -> f32 {
        if data.len() <= forecast_idx + self.price_period {
            return 0.0;
        }
        
        // 计算压力位 (使用最近N天的最高价)
        let mut max_price = f32::MIN;
        for i in 0..self.price_period {
            if forecast_idx + i >= data.len() {
                break;
            }
            
            max_price = max_price.max(data[forecast_idx + i].high);
        }
        
        // 计算当前价格与压力位的距离比例
        let current_price = data[forecast_idx].close;
        if current_price <= 0.0 || max_price <= current_price {
            return 0.0;
        }
        
        // 返回距离比例，值越大表示距离压力位越远
        (max_price - current_price) / current_price
    }
}
