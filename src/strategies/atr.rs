use egostrategy_datahub::models::stock::DailyData as DailyBar;
use crate::stock::indicators::{calculate_atr, standard_deviation, moving_average, extract_price_data};
use super::StockSelector;

/// ATR策略的特征提取结果
#[derive(Debug, Clone)]
pub struct AtrFeatures {
    pub date: String,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub atr: f32,
    pub amplitude: f32,
    pub hist_vol: f32,
    pub mean_vol: f32,
    pub mean_amt: f32,
    pub volume_ratio: f32,
}

/// 从历史数据中提取ATR相关特征
pub fn extract_atr_features(history: &[DailyBar]) -> AtrFeatures {
    let (_opens, highs, lows, closes, volumes, amounts) = extract_price_data(history);
    
    // 获取最新一天的数据
    let last = history.last().unwrap();
    
    // 计算ATR
    let atr_values = calculate_atr(&highs, &lows, &closes, 14);
    let atr = *atr_values.last().unwrap_or(&0.0);
    
    // 计算振幅
    let amplitude = if closes.len() > 1 {
        (highs.last().unwrap() - lows.last().unwrap()) / closes[closes.len() - 2].max(1.0)
    } else {
        0.0
    };
    
    // 计算历史波动率
    let hist_vol = standard_deviation(&closes);
    
    // 计算成交量均值
    let mean_vol = moving_average(&volumes, 5).last().unwrap_or(&0.0).to_owned();
    
    // 计算成交额均值
    let mean_amt = moving_average(&amounts, 5).last().unwrap_or(&0.0).to_owned();
    
    // 计算量比
    let volume_ratio = if mean_vol > 1.0 {
        volumes.last().unwrap() / mean_vol
    } else {
        0.0
    };
    
    AtrFeatures {
        date: last.date.to_string(),
        open: last.open,
        high: last.high,
        low: last.low,
        close: last.close,
        atr,
        amplitude,
        hist_vol,
        mean_vol,
        mean_amt,
        volume_ratio,
    }
}

/// ATR策略的权重配置
#[derive(Debug, Clone)]
pub struct ScoreWeights {
    pub volatility: f32,
    pub liquidity: f32,
    pub trend: f32,
    pub sentiment: f32,
    pub risk: f32,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            volatility: 0.4,
            liquidity: 0.2,
            trend: 0.2,
            sentiment: 0.1,
            risk: 0.1,
        }
    }
}

/// 基于ATR的选股策略
pub struct AtrSelector {
    pub top_n: usize,
    pub lookback_days: usize,
    pub score_weights: ScoreWeights,
}

impl Default for AtrSelector {
    fn default() -> Self {
        Self {
            top_n: 10,
            lookback_days: 100,
            score_weights: ScoreWeights::default(),
        }
    }
}

/// 计算股票得分
pub fn calculate_atr_score(features: &AtrFeatures, weights: &ScoreWeights) -> f32 {
    // 归一化处理
    let volatility = (features.atr * 20.0 + features.amplitude * 100.0).min(100.0);
    let liquidity = (features.volume_ratio * 50.0).min(100.0);
    let trend = 60.0; // 可自定义趋势指标
    let sentiment = 50.0; // 可自定义情绪指标
    let risk = 100.0 - features.hist_vol.min(80.0);
    
    // 加权计算总分
    weights.volatility * volatility +
    weights.liquidity * liquidity +
    weights.trend * trend +
    weights.sentiment * sentiment +
    weights.risk * risk
}

impl StockSelector for AtrSelector {
    fn name(&self) -> String {
        String::from("ATR波动选股策略")
    }
    
    fn top_n(&self) -> usize {
        self.top_n
    }
    
    fn calculate_score(&self, symbol: &str, data: &[DailyBar], forecast_idx: usize) -> f32 {
        if data.len() < self.lookback_days + forecast_idx {
            log::debug!("股票 {}: 数据不足，无法计算分数", symbol);
            return 0.0;
        }
        
        let start = data.len().saturating_sub(self.lookback_days + forecast_idx);
        let end = start + self.lookback_days;
        
        if end > data.len() {
            log::debug!("股票 {}: 索引超出范围 (start={}, end={}, len={})", 
                symbol, start, end, data.len());
            return 0.0;
        }
        
        let history = &data[start..end];
        log::debug!("股票 {}: 使用历史数据 {} 条记录 (start={}, end={})", 
            symbol, history.len(), start, end);
            
        let features = extract_atr_features(history);
        let score = calculate_atr_score(&features, &self.score_weights);
        
        log::debug!("股票 {}: 计算得分 = {:.2}, ATR = {:.4}, 振幅 = {:.2}%, 量比 = {:.2}", 
            symbol, score, features.atr, features.amplitude * 100.0, features.volume_ratio);
            
        score
    }
}
