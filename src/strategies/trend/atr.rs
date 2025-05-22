use crate::strategies::StockSelector;
use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// ATR选股策略的权重配置
#[derive(Debug, Clone)]
pub struct AtrSelectorWeights {
    pub atr_weight: f32,
    pub volume_weight: f32,
    pub trend_weight: f32,
}

impl Default for AtrSelectorWeights {
    fn default() -> Self {
        Self {
            atr_weight: 0.4,
            volume_weight: 0.3,
            trend_weight: 0.3,
        }
    }
}

/// 基于ATR的选股策略
#[derive(Debug, Clone)]
pub struct AtrSelector {
    pub top_n: usize,
    pub lookback_days: usize,
    pub score_weights: AtrSelectorWeights,
}

impl StockSelector for AtrSelector {
    fn name(&self) -> String {
        "ATR选股策略".to_string()
    }
    
    fn run(&self, stock_data: &[(String, Vec<DailyBar>)], forecast_idx: usize) -> Vec<(String, Vec<DailyBar>)> {
        // 计算每只股票的得分
        let mut scores = Vec::new();
        
        for (symbol, data) in stock_data {
            if data.len() <= forecast_idx + self.lookback_days {
                continue;
            }
            
            // 计算ATR
            let atr = self.calculate_atr(data, forecast_idx);
            
            // 计算成交量得分
            let volume_score = self.calculate_volume_score(data, forecast_idx);
            
            // 计算趋势得分
            let trend_score = self.calculate_trend_score(data, forecast_idx);
            
            // 计算总得分
            let total_score = 
                atr * self.score_weights.atr_weight + 
                volume_score * self.score_weights.volume_weight + 
                trend_score * self.score_weights.trend_weight;
                
            scores.push((symbol.clone(), data.clone(), total_score));
        }
        
        // 按得分排序
        scores.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        
        // 取前N名
        scores.into_iter()
            .take(self.top_n)
            .map(|(symbol, data, _)| (symbol, data))
            .collect()
    }
}

impl AtrSelector {
    /// 计算ATR (Average True Range)
    fn calculate_atr(&self, data: &[DailyBar], forecast_idx: usize) -> f32 {
        if data.len() <= forecast_idx + 1 {
            return 0.0;
        }
        
        let mut tr_sum = 0.0;
        let period = self.lookback_days.min(data.len() - forecast_idx - 1);
        
        for i in 0..period {
            let idx = forecast_idx + i;
            let prev_idx = idx + 1;
            
            if prev_idx >= data.len() {
                continue;
            }
            
            // 计算真实波动幅度 (True Range)
            let high_low = data[idx].high - data[idx].low;
            let high_prev_close = (data[idx].high - data[prev_idx].close).abs();
            let low_prev_close = (data[idx].low - data[prev_idx].close).abs();
            
            let tr = high_low.max(high_prev_close).max(low_prev_close);
            tr_sum += tr;
        }
        
        // 计算ATR
        let atr = if period > 0 {
            tr_sum / period as f32
        } else {
            0.0
        };
        
        // 归一化ATR (相对于价格)
        let price = data[forecast_idx].close;
        if price > 0.0 {
            atr / price
        } else {
            0.0
        }
    }
    
    /// 计算成交量得分
    fn calculate_volume_score(&self, data: &[DailyBar], forecast_idx: usize) -> f32 {
        if data.len() <= forecast_idx + self.lookback_days {
            return 0.0;
        }
        
        // 计算最近N天的平均成交量
        let mut volume_sum = 0.0;
        let period = self.lookback_days.min(data.len() - forecast_idx);
        
        for i in 0..period {
            volume_sum += data[forecast_idx + i].volume as f32;
        }
        
        let avg_volume = if period > 0 {
            volume_sum / period as f32
        } else {
            0.0
        };
        
        // 计算最近一天的成交量相对于平均值的比例
        if avg_volume > 0.0 {
            data[forecast_idx].volume as f32 / avg_volume
        } else {
            0.0
        }
    }
    
    /// 计算趋势得分
    fn calculate_trend_score(&self, data: &[DailyBar], forecast_idx: usize) -> f32 {
        if data.len() <= forecast_idx + self.lookback_days {
            return 0.0;
        }
        
        // 计算最近N天的价格变化率
        let start_price = data[forecast_idx + self.lookback_days - 1].close;
        let end_price = data[forecast_idx].close;
        
        if start_price > 0.0 {
            (end_price - start_price) / start_price
        } else {
            0.0
        }
    }
}
