pub mod trend;
pub mod reversal;
pub mod volume;

use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 选股策略特征
pub trait StockSelector: Send + Sync {
    /// 获取策略名称
    fn name(&self) -> String;
    
    /// 运行选股策略
    fn run(&self, stock_data: &[(String, Vec<DailyBar>)], forecast_idx: usize) -> Vec<(String, Vec<DailyBar>)>;
}
