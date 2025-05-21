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
            log::debug!("评估失败: 买入价格无效 ({})", buy_price);
            return false;
        }
        
        // 对于倒序数据，forecast_idx表示从最新数据往后数的天数
        // 我们需要检查从forecast_idx+1到forecast_idx+in_days的数据
        if data.len() <= forecast_idx + self.in_days {
            log::debug!("评估失败: 数据不足 (需要 {} 天, 实际 {} 天)", 
                forecast_idx + self.in_days, data.len());
            return false;
        }
        
        log::debug!("评估区间: 从idx={}到idx={}, 买入价={:.2}", 
            forecast_idx + 1, forecast_idx + self.in_days, buy_price);
        
        // 检查区间内是否触发止损
        for i in (forecast_idx + 1)..=(forecast_idx + self.in_days) {
            let low_return = (data[i].low - buy_price) / buy_price;
            if low_return <= -self.stop_loss {
                log::debug!("止损触发: 第{}天亏损{:.2}% (止损{:.2}%)", 
                    i - forecast_idx, -low_return * 100.0, self.stop_loss * 100.0);
                return false;
            }
        }
        
        log::debug!("目标达成: {}天内未触发{:.2}%止损", self.in_days, self.stop_loss * 100.0);
        true
    }
}
