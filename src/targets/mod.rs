pub mod return_target;
pub mod guard_target;

use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 回测目标特征
pub trait Target: Sync + Send {
    /// 返回目标名称
    fn name(&self) -> String;
    
    /// 返回目标评估天数
    fn in_days(&self) -> usize;
    
    /// 返回目标收益率
    fn target_return(&self) -> f32;
    
    /// 返回止损比例
    fn stop_loss(&self) -> f32;
    
    /// 评估目标是否达成
    fn evaluate(&self, data: &[DailyBar], buy_price: f32, forecast_idx: usize) -> bool;
    
    /// 运行目标评估，返回达成率
    fn run(&self, candidates: Vec<(String, Vec<DailyBar>, f32)>, forecast_idx: usize) -> f32 {
        if candidates.is_empty() {
            return 0.0;
        }
        
        let success_count = candidates
            .iter()
            .filter(|(_, data, buy_price)| self.evaluate(data, *buy_price, forecast_idx))
            .count();
            
        success_count as f32 / candidates.len() as f32
    }
}
