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
        
        // 检查区间内的最高价是否达到目标收益
        for i in (forecast_idx + 1)..=(forecast_idx + self.in_days) {
            let current_return = (data[i].high - buy_price) / buy_price;
            if current_return >= self.target_return {
                log::debug!("目标达成: 第{}天达到收益率{:.2}% (目标{:.2}%)", 
                    i - forecast_idx, current_return * 100.0, self.target_return * 100.0);
                return true;
            }
            
            // 检查是否触发止损
            let low_return = (data[i].low - buy_price) / buy_price;
            if low_return <= -self.stop_loss {
                log::debug!("止损触发: 第{}天亏损{:.2}% (止损{:.2}%)", 
                    i - forecast_idx, -low_return * 100.0, self.stop_loss * 100.0);
                return false;
            }
        }
        
        log::debug!("目标未达成: 未在{}天内达到{:.2}%收益率", self.in_days, self.target_return * 100.0);
        false
    }
}
