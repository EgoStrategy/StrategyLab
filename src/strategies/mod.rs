pub mod atr;
pub mod macd;
pub mod rsi;
pub mod volume_decline;
pub mod breakthrough_pullback;  // 添加新模块

use rayon::prelude::*;
use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 选股策略特征
pub trait StockSelector: Sync + Send {
    /// 返回策略名称
    fn name(&self) -> String;
    
    /// 返回选择的股票数量
    fn top_n(&self) -> usize;
    
    /// 计算股票分数，用于排名
    fn calculate_score(&self, symbol: &str, data: &[DailyBar], forecast_idx: usize) -> f32;
    
    /// 运行选股策略，返回候选股票列表
    fn run(&self, stock_data: &[(String, Vec<DailyBar>)], forecast_idx: usize) -> Vec<(String, Vec<DailyBar>)> {
        log::info!("运行选股策略: {}, 预测天数={}", self.name(), forecast_idx);
        
        if stock_data.is_empty() {
            log::warn!("没有股票数据可供选择");
            return Vec::new();
        }
        
        log::info!("计算 {} 只股票的分数", stock_data.len());
        
        let mut scores: Vec<(&String, &Vec<DailyBar>, f32)> = stock_data
            .par_iter()
            .map(|(symbol, data)| {
                let score = self.calculate_score(symbol, data, forecast_idx);
                (symbol, data, score)
            })
            .collect();
            
        scores.par_sort_unstable_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        
        let candidates: Vec<(String, Vec<DailyBar>)> = scores.into_iter()
            .take(self.top_n())
            .map(|(symbol, data, score)| {
                log::debug!("选中股票: {}, 分数: {:.2}", symbol, score);
                (symbol.clone(), data.clone())
            })
            .collect();
            
        log::info!("选股完成: 选出 {} 只候选股票", candidates.len());
        candidates
    }
}

// 导出新的选股器
pub use self::breakthrough_pullback::BreakthroughPullbackSelector;
