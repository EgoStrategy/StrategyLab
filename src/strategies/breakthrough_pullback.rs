use crate::strategies::StockSelector;
use egostrategy_datahub::models::stock::DailyData as DailyBar;
use log::debug;

/// 突破回踩选股器 - 结合打板与地包天特点的进攻性策略
pub struct BreakthroughPullbackSelector {
    pub top_n: usize,
    pub lookback_days: usize,
    pub min_breakthrough_percent: f32,  // 突破幅度最小值
    pub max_pullback_percent: f32,      // 回调幅度最大值
    pub volume_decline_ratio: f32,      // 量能萎缩比例
}

impl Default for BreakthroughPullbackSelector {
    fn default() -> Self {
        Self {
            top_n: 10,
            lookback_days: 10,
            min_breakthrough_percent: 5.0,
            max_pullback_percent: 5.0,
            volume_decline_ratio: 0.7,
        }
    }
}

impl StockSelector for BreakthroughPullbackSelector {
    fn name(&self) -> String {
        format!("突破回踩策略")
    }
    
    fn top_n(&self) -> usize {
        self.top_n
    }
    
    fn calculate_score(&self, symbol: &str, data: &[DailyBar], forecast_idx: usize) -> f32 {
        if data.len() < forecast_idx + self.lookback_days + 5 {
            return 0.0;  // 确保有足够的历史数据
        }
        
        // 计算相关指标
        let mut found_breakthrough = false;
        let mut breakthrough_idx = 0;
        let mut breakthrough_price = 0.0;
        
        // 1. 寻找近期突破（涨停或大阳线）
        for i in (forecast_idx + 1)..=(forecast_idx + self.lookback_days) {
            if i >= data.len() || i == 0 {
                continue;
            }
            
            let prev_close = data[i-1].close;
            let curr_close = data[i].close;
            let percent_change = (curr_close - prev_close) / prev_close * 100.0;
            
            // 判断是否为涨停或大阳线
            if percent_change >= self.min_breakthrough_percent {
                found_breakthrough = true;
                breakthrough_idx = i;
                breakthrough_price = curr_close;
                break;
            }
        }
        
        if !found_breakthrough {
            return 0.0;  // 没有找到突破，跳过
        }
        
        // 2. 检查突破后的回调
        let mut found_pullback = false;
        let mut current_low = f32::MAX;
        let mut volume_declined = false;
        
        for i in (forecast_idx + 1)..breakthrough_idx {
            if i >= data.len() {
                continue;
            }
            
            // 更新最低价
            current_low = current_low.min(data[i].low);
            
            // 计算回调幅度
            let pullback_percent = (breakthrough_price - current_low) / breakthrough_price * 100.0;
            
            // 检查量能是否萎缩
            let breakthrough_volume = data[breakthrough_idx].volume;
            let current_volume = data[i].volume;
            volume_declined = current_volume as f32 <= breakthrough_volume as f32 * self.volume_decline_ratio;
            
            // 判断是否满足回调条件
            if pullback_percent <= self.max_pullback_percent && pullback_percent > 0.0 && volume_declined {
                found_pullback = true;
                break;
            }
        }
        
        if !found_pullback {
            return 0.0;  // 没有找到合适的回调，跳过
        }
        
        // 3. 检查MACD指标（简化版，仅检查DIF和DEA的关系）
        let mut macd_golden_cross = false;
        
        // 计算最近的EMA12和EMA26
        let mut ema12 = 0.0;
        let mut ema26 = 0.0;
        let mut prev_ema12 = 0.0;
        let mut prev_ema26 = 0.0;
        
        // 简化的MACD计算
        for i in (forecast_idx + 1)..=(forecast_idx + 30).min(data.len() - 1) {
            if i == forecast_idx + 1 {
                // 初始化
                ema12 = data[i].close;
                ema26 = data[i].close;
            } else {
                prev_ema12 = ema12;
                prev_ema26 = ema26;
                
                // 更新EMA
                ema12 = prev_ema12 * 11.0 / 13.0 + data[i].close * 2.0 / 13.0;
                ema26 = prev_ema26 * 25.0 / 27.0 + data[i].close * 2.0 / 27.0;
                
                // 检查是否金叉
                let prev_dif = prev_ema12 - prev_ema26;
                let curr_dif = ema12 - ema26;
                
                if prev_dif < 0.0 && curr_dif >= 0.0 {
                    macd_golden_cross = true;
                    break;
                }
            }
        }
        
        // 4. 综合判断
        if found_breakthrough && found_pullback && (macd_golden_cross || volume_declined) {
            debug!("选中股票 {}: 突破={:.2}%, 回调={:.2}%, MACD金叉={}, 量能萎缩={}",
                symbol, self.min_breakthrough_percent, self.max_pullback_percent, macd_golden_cross, volume_declined);
            
            // 计算分数 - 基于突破幅度和回调程度
            let breakthrough_score = breakthrough_price / data[breakthrough_idx + 1].close - 1.0;
            let pullback_score = 1.0 - (current_low / breakthrough_price);
            let macd_score = if macd_golden_cross { 1.0 } else { 0.5 };
            
            return (breakthrough_score * 50.0 + pullback_score * 30.0 + macd_score * 20.0) * 100.0;
        }
        
        0.0
    }
}
