use egostrategy_datahub::models::stock::DailyData as DailyBar;
use crate::stock::indicators::{calculate_atr, standard_deviation, extract_price_data};
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

/// 从历史数据中提取ATR相关特征 - 适用于倒序数据
pub fn extract_atr_features(history: &[DailyBar]) -> AtrFeatures {
    let (_opens, highs, lows, closes, volumes, amounts) = extract_price_data(history);
    
    // 获取最新一天的数据（倒序数据中的第一个）
    let last = &history[0];
    
    // 计算ATR
    let atr_values = calculate_atr(&highs, &lows, &closes, 14);
    let atr = atr_values[0]; // 最新的ATR值（倒序数据中的第一个）
    
    // 计算振幅
    let amplitude = if history.len() > 1 {
        (highs[0] - lows[0]) / closes[1].max(1.0)
    } else {
        0.0
    };
    
    // 计算历史波动率
    let hist_vol = standard_deviation(&closes);
    
    // 计算成交量均值（最近5天）
    let vol_lookback = 5.min(history.len());
    let mut mean_vol = 0.0;
    for i in 0..vol_lookback {
        mean_vol += volumes[i];
    }
    mean_vol /= vol_lookback as f32;
    
    // 计算成交额均值（最近5天）
    let mut mean_amt = 0.0;
    for i in 0..vol_lookback {
        mean_amt += amounts[i];
    }
    mean_amt /= vol_lookback as f32;
    
    // 计算量比
    let volume_ratio = if mean_vol > 1.0 {
        volumes[0] / mean_vol
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
        // 对于倒序数据，forecast_idx表示从最新数据往后数的天数
        if data.len() <= forecast_idx || data.len() < self.lookback_days {
            log::debug!("股票 {}: 数据不足，无法计算分数", symbol);
            return 0.0;
        }
        
        // 对于倒序数据，我们需要从forecast_idx开始取lookback_days天的数据
        let end = forecast_idx + self.lookback_days;
        if end > data.len() {
            log::debug!("股票 {}: 索引超出范围 (forecast_idx={}, lookback_days={}, len={})", 
                symbol, forecast_idx, self.lookback_days, data.len());
            return 0.0;
        }
        
        let history = &data[forecast_idx..end];
        log::debug!("股票 {}: 使用历史数据 {} 条记录 (forecast_idx={}, end={})", 
            symbol, history.len(), forecast_idx, end);
            
        let features = extract_atr_features(history);
        let score = calculate_atr_score(&features, &self.score_weights);
        
        log::debug!("股票 {}: 计算得分 = {:.2}, ATR = {:.4}, 振幅 = {:.2}%, 量比 = {:.2}", 
            symbol, score, features.atr, features.amplitude * 100.0, features.volume_ratio);
            
        score
    }
}
