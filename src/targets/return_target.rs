use egostrategy_datahub::models::stock::DailyData as DailyBar;
use super::Target;

/// 收益率目标
pub struct ReturnTarget {
    pub target_return: f32,  // 目标收益率
    pub stop_loss: f32,      // 止损比例
    pub in_days: usize,      // 目标天数
}

impl ReturnTarget {
    /// 创建新的收益率目标
    pub fn new(target_return: f32, stop_loss: f32, in_days: usize) -> Self {
        Self {
            target_return,
            stop_loss,
            in_days,
        }
    }
}

impl Target for ReturnTarget {
    fn name(&self) -> String {
        format!("{}天内收益率达到{}%", self.in_days, self.target_return * 100.0)
    }
    
    fn in_days(&self) -> usize {
        self.in_days
    }
    
    fn target_return(&self) -> f32 {
        self.target_return
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
        
        // 检查区间内的最高价是否达到目标收益
        for i in end_idx..start_idx {
            let current_return = (data[i].high - buy_price) / buy_price;
            if current_return >= self.target_return {
                return true;
            }
            
            // 检查是否触发止损
            let low_return = (data[i].low - buy_price) / buy_price;
            if low_return <= -self.stop_loss {
                return false;
            }
        }
        
        false
    }
}
