use egostrategy_datahub::models::stock::DailyData as DailyBar;
use super::Target;

/// 止损目标
pub struct GuardTarget {
    pub stop_loss: f32,      // 止损比例
    pub in_days: usize,      // 目标天数
}

impl GuardTarget {
    /// 创建新的止损目标
    pub fn new(stop_loss: f32, in_days: usize) -> Self {
        Self {
            stop_loss,
            in_days,
        }
    }
}

impl Target for GuardTarget {
    fn name(&self) -> String {
        format!("{}天内不触发{}%止损", self.in_days, self.stop_loss * 100.0)
    }
    
    fn in_days(&self) -> usize {
        self.in_days
    }
    
    fn target_return(&self) -> f32 {
        0.0 // 没有目标收益率
    }
    
    fn stop_loss(&self) -> f32 {
        self.stop_loss
    }
    
    fn evaluate(&self, data: &[DailyBar], buy_price: f32, forecast_idx: usize) -> bool {
        if buy_price <= 0.0 {
            return false;
        }
        
        let start_idx = data.len().saturating_sub(forecast_idx);
        let end_idx = start_idx.saturating_sub(self.in_days);
        
        if end_idx >= start_idx || end_idx >= data.len() {
            return false;
        }
        
        // 检查区间内是否触发止损
        for i in end_idx..start_idx {
            let low_return = (data[i].low - buy_price) / buy_price;
            if low_return <= -self.stop_loss {
                return false;
            }
        }
        
        true
    }
}
