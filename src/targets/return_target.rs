use crate::targets::Target;
use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 收益率目标
#[derive(Debug, Clone)]
pub struct ReturnTarget {
    pub target_return: f32,
    pub stop_loss: f32,
    pub in_days: usize,
}

impl Target for ReturnTarget {
    fn name(&self) -> String {
        format!("收益率目标 {}% / {}天", self.target_return * 100.0, self.in_days)
    }
    
    fn target_return(&self) -> f32 {
        self.target_return
    }
    
    fn stop_loss(&self) -> f32 {
        self.stop_loss
    }
    
    fn in_days(&self) -> usize {
        self.in_days
    }
    
    fn run(&self, signals: Vec<(String, Vec<DailyBar>, f32)>, forecast_idx: usize) -> f32 {
        let mut total_trades = signals.len();
        let mut winning_trades = 0;
        
        for (_, data, buy_price) in signals {
            if buy_price <= 0.0 {
                total_trades -= 1;
                continue;
            }
            
            // 对于倒序数据，forecast_idx表示从最新数据往后数的天数
            // 我们需要检查从forecast_idx+1到forecast_idx+in_days的数据
            if data.len() <= forecast_idx + self.in_days {
                total_trades -= 1;
                continue;
            }
            
            // 计算止损价
            let stop_loss_price = buy_price * (1.0 - self.stop_loss);
            
            // 检查是否达到目标收益或触发止损
            let mut is_win = false;
            
            // 检查第一个交易日是否直接低于止损价（止损失败）
            if data[forecast_idx + 1].open < stop_loss_price {
                // 止损失败，直接计为失败
                is_win = false;
            } else {
                // 正常交易流程
                for i in (forecast_idx + 1)..=(forecast_idx + self.in_days) {
                    // 检查是否达到目标收益
                    if (data[i].close - buy_price) / buy_price >= self.target_return {
                        is_win = true;
                        break;
                    }
                    
                    // 检查是否跳空低开导致止损失败
                    if i > forecast_idx + 1 && data[i].open < stop_loss_price {
                        // 止损失败，直接计为失败
                        is_win = false;
                        break;
                    }
                    
                    // 检查是否触发正常止损
                    if data[i].low <= stop_loss_price && data[i].open >= stop_loss_price {
                        // 正常止损，计为失败
                        is_win = false;
                        break;
                    }
                }
            }
            
            if is_win {
                winning_trades += 1;
            }
        }
        
        if total_trades > 0 {
            winning_trades as f32 / total_trades as f32
        } else {
            0.0
        }
    }
}
