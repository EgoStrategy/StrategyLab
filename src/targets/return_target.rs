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
        let (total_trades, winning_trades, _, _, _, _) = self.evaluate_signals(signals, forecast_idx);
        
        if total_trades > 0 {
            winning_trades as f32 / total_trades as f32
        } else {
            0.0
        }
    }
    
    fn evaluate_signals(&self, signals: Vec<(String, Vec<DailyBar>, f32)>, forecast_idx: usize) 
        -> (usize, usize, usize, usize, Vec<f32>, Vec<f32>) {
        let mut total_trades = signals.len();
        let mut winning_trades = 0;
        let mut losing_trades = 0;
        let mut stop_loss_trades = 0;
        let mut returns = Vec::new();
        let mut hold_days = Vec::new();
        
        for (_, data, buy_price) in signals {
            if buy_price <= 0.0 {
                total_trades -= 1;
                continue;
            }
            
            // 确保有足够的历史数据进行回测
            if forecast_idx < self.in_days || data.len() <= forecast_idx {
                total_trades -= 1;
                continue;
            }
            
            // 计算最大收益和止损
            let mut max_return = -1.0;
            let mut exit_day = 0;
            let mut is_win = false;
            let mut is_stop_loss = false;
            
            // 检查从forecast_idx-self.in_days到forecast_idx-1的数据
            for i in (forecast_idx - self.in_days)..forecast_idx {
                // 先检查收盘价是否触发止损
                let current_return = (data[i].close - buy_price) / buy_price;
                
                // 如果亏损超过止损的2倍，认为是止损失败
                if current_return < -2.0 * self.stop_loss {
                    // 不再使用is_stop_loss_fail
                    max_return = current_return;
                    exit_day = i - (forecast_idx - self.in_days) + 1;
                    break;
                }
                // 如果亏损超过止损线，认为是正常止损
                else if current_return < -self.stop_loss {
                    is_stop_loss = true;
                    max_return = current_return;
                    exit_day = i - (forecast_idx - self.in_days) + 1;
                    break;
                }
                // 如果达到目标收益，认为是成功
                else if current_return >= self.target_return {
                    is_win = true;
                    max_return = current_return;
                    exit_day = i - (forecast_idx - self.in_days) + 1;
                    break;
                }
                
                // 更新最大收益
                if current_return > max_return {
                    max_return = current_return;
                }
            }
            
            // 如果没有提前退出，使用最后一天的收盘价计算收益
            if exit_day == 0 {
                let last_idx = forecast_idx - 1;
                let last_return = (data[last_idx].close - buy_price) / buy_price;
                max_return = last_return;
                exit_day = self.in_days;
                
                // 检查最后一天是否达到目标收益
                if last_return >= self.target_return {
                    is_win = true;
                }
                // 检查最后一天是否触发止损
                else if last_return < -self.stop_loss {
                    if last_return < -2.0 * self.stop_loss {
                        // 不再使用is_stop_loss_fail
                    } else {
                        is_stop_loss = true;
                    }
                }
            }
            
            // 统计结果
            if is_win {
                winning_trades += 1;
            } else {
                losing_trades += 1;
                if is_stop_loss {
                    stop_loss_trades += 1;
                }
                // 不再统计止损失败交易数
            }
            
            returns.push(max_return);
            hold_days.push(exit_day as f32);
        }
        
        (total_trades, winning_trades, losing_trades, stop_loss_trades, returns, hold_days)
    }
}
