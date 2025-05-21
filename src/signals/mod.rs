pub mod price;
pub mod volume;

use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 买入信号生成器特征
pub trait BuySignalGenerator: Sync + Send {
    /// 返回信号生成器名称
    fn name(&self) -> String;
    
    /// 计算买入价格
    fn calculate_buy_price(&self, symbol: &str, data: &[DailyBar], forecast_idx: usize) -> f32;
    
    /// 生成买入信号
    fn generate_signals(&self, candidates: Vec<(String, Vec<DailyBar>)>, forecast_idx: usize) -> Vec<(String, Vec<DailyBar>, f32)> {
        candidates
            .into_iter()
            .map(|(symbol, data)| {
                let buy_price = self.calculate_buy_price(&symbol, &data, forecast_idx);
                (symbol, data, buy_price)
            })
            .collect()
    }
}
