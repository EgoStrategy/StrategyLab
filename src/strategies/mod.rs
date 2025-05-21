pub mod atr;
pub mod macd;
pub mod rsi;

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
        let mut scores: Vec<(&String, &Vec<DailyBar>, f32)> = stock_data
            .par_iter()
            .map(|(symbol, data)| {
                (symbol, data, self.calculate_score(symbol, data, forecast_idx))
            })
            .collect();
            
        scores.par_sort_unstable_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        
        scores.into_iter()
            .take(self.top_n())
            .map(|(symbol, data, _)| (symbol.clone(), data.clone()))
            .collect()
    }
}
