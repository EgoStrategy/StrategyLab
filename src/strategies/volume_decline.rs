use egostrategy_datahub::models::stock::DailyData as DailyBar;
use crate::stock::indicators::{extract_price_data};
use super::StockSelector;
use std::cmp::Ordering;

/// 连续下跌缩量策略的特征提取结果
#[derive(Debug, Clone)]
pub struct VolumeDecliningFeatures {
    pub date: String,
    pub close: f32,
    pub support_level: f32,
    pub resistance_level: f32,
    pub distance_to_resistance: f32,  // 距离压力位的比例
    pub volume_decline_ratio: f32,    // 成交量缩减比例
    pub consecutive_decline_days: i32, // 连续下跌天数
}

/// 从历史数据中提取连续下跌缩量相关特征
pub fn extract_volume_declining_features(
    history: &[DailyBar], 
    min_consecutive_decline_days: i32,
    min_volume_decline_ratio: f32,
    price_period: usize
) -> Option<VolumeDecliningFeatures> {
    if history.len() < 10 {
        return None;
    }
    
    let (_opens, highs, lows, _closes, volumes, _amounts) = extract_price_data(history);
    
    // 获取最新一天的数据
    let last = history.last().unwrap();
    
    // 计算连续下跌天数
    let mut consecutive_decline_days = 0;
    for i in (1..5).rev() {
        if i >= history.len() {
            continue;
        }
        let idx = history.len() - 1 - i;
        if idx + 1 < history.len() && history[idx].close > history[idx + 1].close {
            consecutive_decline_days += 1;
        } else {
            break;
        }
    }
    
    // 如果连续下跌天数不足要求，则不符合条件
    if consecutive_decline_days < min_consecutive_decline_days {
        return None;
    }
    
    // 计算支撑位和压力位
    // 使用过去N天的数据计算
    let period = price_period.min(history.len() - 1);
    let start_idx = history.len().saturating_sub(period);
    let end_idx = history.len() - 1;
    
    let mut support_level = f32::MAX;
    let mut resistance_level: f32 = 0.0;
    
    for i in start_idx..=end_idx {
        support_level = support_level.min(lows[i]);
        resistance_level = resistance_level.max(highs[i]);
    }
    
    // 检查是否破位
    if last.close < support_level {
        return None; // 已经破位，不符合条件
    }
    
    // 计算成交量缩减比例
    // 使用5日平均成交量作为基准
    let vol_lookback = 5.min(end_idx);
    let vol_start = end_idx.saturating_sub(vol_lookback);
    
    if vol_start >= end_idx {
        return None;
    }
    
    let avg_volume = if end_idx > vol_lookback {
        let vol_slice = &volumes[vol_start..end_idx];
        vol_slice.iter().sum::<f32>() / vol_slice.len() as f32
    } else {
        volumes[end_idx] // 如果数据不足，使用当前成交量
    };
    
    let current_volume = volumes[end_idx];
    let volume_decline_ratio = if avg_volume > 0.0 {
        1.0 - current_volume / avg_volume
    } else {
        0.0
    };
    
    // 如果成交量没有明显缩减，则不符合条件
    if volume_decline_ratio < min_volume_decline_ratio {
        return None;
    }
    
    // 计算当前价格距离压力位的比例
    let distance_to_resistance = if last.close < resistance_level {
        (resistance_level - last.close) / last.close
    } else {
        0.0
    };
    
    Some(VolumeDecliningFeatures {
        date: last.date.to_string(),
        close: last.close,
        support_level,
        resistance_level,
        distance_to_resistance,
        volume_decline_ratio,
        consecutive_decline_days,
    })
}

/// 连续下跌缩量选股策略
pub struct VolumeDecliningSelector {
    pub top_n: usize,                    // 选出的股票数量
    pub lookback_days: usize,            // 回看的历史数据天数
    pub min_consecutive_decline_days: i32, // 最少连续下跌天数
    pub min_volume_decline_ratio: f32,   // 最小成交量缩减比例
    pub price_period: usize,             // 计算支撑位和压力位的周期
    pub check_support_level: bool,       // 是否检查支撑位
}

impl Default for VolumeDecliningSelector {
    fn default() -> Self {
        Self {
            top_n: 10,
            lookback_days: 30,
            min_consecutive_decline_days: 2,  // 默认要求连续2天下跌
            min_volume_decline_ratio: 0.1,    // 默认要求成交量缩减10%
            price_period: 20,                 // 默认使用20天数据计算支撑压力位
            check_support_level: false,       // 默认不检查是否破位
        }
    }
}

impl StockSelector for VolumeDecliningSelector {
    fn name(&self) -> String {
        String::from("连续下跌缩量策略")
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
        let end = data.len() - forecast_idx;
        
        if end <= start {
            log::debug!("股票 {}: 索引范围无效 (start={}, end={})", symbol, start, end);
            return 0.0;
        }
        
        let history = &data[start..end];
        
        // 提取特征
        match extract_volume_declining_features(
            history, 
            self.min_consecutive_decline_days,
            self.min_volume_decline_ratio,
            self.price_period
        ) {
            Some(features) => {
                // 计算分数 - 主要基于距离压力位的比例
                let distance_score = features.distance_to_resistance * 100.0;
                let volume_score = features.volume_decline_ratio * 50.0;
                
                let total_score = distance_score + volume_score;
                
                log::debug!("股票 {}: 连续下跌{}天, 缩量比例={:.2}%, 距压力位={:.2}%, 总分={:.2}", 
                    symbol, 
                    features.consecutive_decline_days,
                    features.volume_decline_ratio * 100.0,
                    features.distance_to_resistance * 100.0,
                    total_score);
                
                total_score
            },
            None => {
                log::debug!("股票 {}: 不符合连续下跌缩量条件", symbol);
                0.0
            }
        }
    }
    
    /// 重写run方法，按照距离压力位的比例排序
    fn run(&self, stock_data: &[(String, Vec<DailyBar>)], forecast_idx: usize) -> Vec<(String, Vec<DailyBar>)> {
        log::info!("运行选股策略: {}, 预测天数={}", self.name(), forecast_idx);
        
        if stock_data.is_empty() {
            log::warn!("没有股票数据可供选择");
            return Vec::new();
        }
        
        log::info!("计算 {} 只股票的分数", stock_data.len());
        
        // 提取所有符合条件的股票及其特征
        let mut candidates = Vec::new();
        
        for (symbol, data) in stock_data {
            if data.len() < self.lookback_days + forecast_idx {
                continue;
            }
            
            let start = data.len().saturating_sub(self.lookback_days + forecast_idx);
            let end = data.len() - forecast_idx;
            
            if end <= start {
                continue;
            }
            
            let history = &data[start..end];
            
            if let Some(features) = extract_volume_declining_features(
                history, 
                self.min_consecutive_decline_days,
                self.min_volume_decline_ratio,
                self.price_period
            ) {
                candidates.push((symbol, data, features));
            }
        }
        
        log::info!("找到 {} 只符合条件的股票", candidates.len());
        
        // 按照距离压力位的比例排序，距离越远排名越靠前
        candidates.sort_by(|a, b| {
            b.2.distance_to_resistance.partial_cmp(&a.2.distance_to_resistance)
                .unwrap_or(Ordering::Equal)
        });
        
        // 取前N只股票
        let result: Vec<(String, Vec<DailyBar>)> = candidates.into_iter()
            .take(self.top_n())
            .map(|(symbol, data, features)| {
                log::debug!("选中股票: {}, 距压力位: {:.2}%, 缩量比例: {:.2}%, 连续下跌: {}天", 
                    symbol, 
                    features.distance_to_resistance * 100.0,
                    features.volume_decline_ratio * 100.0,
                    features.consecutive_decline_days);
                (symbol.clone(), data.clone())
            })
            .collect();
        
        log::info!("选股完成: 选出 {} 只候选股票", result.len());
        result
    }
}
