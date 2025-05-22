pub mod price;
pub mod pattern;
pub mod volume;

use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 买入信号生成器特征
pub trait BuySignalGenerator: Send + Sync {
    /// 获取信号生成器名称
    fn name(&self) -> String;
    
    /// 生成买入信号
    fn generate_signals(
        &self,
        candidates: Vec<(String, Vec<DailyBar>)>,
        forecast_idx: usize,
    ) -> Vec<(String, Vec<DailyBar>, f32)>;
}
