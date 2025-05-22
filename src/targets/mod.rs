pub mod return_target;
pub mod guard_target;
pub mod combined_target;

use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 目标特征
pub trait Target: Send + Sync {
    /// 获取目标名称
    fn name(&self) -> String;
    
    /// 获取目标收益率
    fn target_return(&self) -> f32;
    
    /// 获取止损比例
    fn stop_loss(&self) -> f32;
    
    /// 获取目标天数
    fn in_days(&self) -> usize;
    
    /// 运行目标评估，返回成功率
    fn run(&self, signals: Vec<(String, Vec<DailyBar>, f32)>, forecast_idx: usize) -> f32;
    
    /// 详细评估信号，返回交易详情
    fn evaluate_signals(&self, signals: Vec<(String, Vec<DailyBar>, f32)>, forecast_idx: usize) 
        -> (usize, usize, usize, usize, Vec<f32>, Vec<f32>);
}
