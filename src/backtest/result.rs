use serde::{Serialize, Deserialize};

/// 交易详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeDetail {
    pub symbol: String,
    pub entry_date: String,
    pub entry_price: f32,
    pub exit_date: String,
    pub exit_price: f32,
    pub return_pct: f32,
    pub hold_days: usize,
    pub exit_reason: ExitReason,
}

/// 退出原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExitReason {
    TargetReached,
    StopLoss,
    StopLossFailed,
    TimeExpired,
}

/// 增强的回测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    // 基本统计
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub stop_loss_trades: usize,
    pub stop_loss_fail_trades: usize,
    
    // 比率
    pub win_rate: f32,
    pub stop_loss_rate: f32,
    pub stop_loss_fail_rate: f32,
    
    // 收益指标
    pub avg_return: f32,
    pub max_return: f32,
    pub max_loss: f32,
    pub avg_hold_days: f32,
    
    // 高级指标
    pub sharpe_ratio: f32,
    pub max_drawdown: f32,
    pub profit_factor: f32,
    
    // 详细交易记录(可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_details: Option<Vec<TradeDetail>>,
}

impl BacktestResult {
    /// 创建新的空结果
    pub fn new() -> Self {
        Self {
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            stop_loss_trades: 0,
            stop_loss_fail_trades: 0,
            win_rate: 0.0,
            stop_loss_rate: 0.0,
            stop_loss_fail_rate: 0.0,
            avg_return: 0.0,
            max_return: 0.0,
            max_loss: 0.0,
            avg_hold_days: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            profit_factor: 0.0,
            trade_details: None,
        }
    }
    
    /// 合并多个回测结果
    pub fn merge(results: Vec<Self>) -> Self {
        if results.is_empty() {
            return Self::new();
        }
        
        let mut total_trades = 0;
        let mut winning_trades = 0;
        let mut losing_trades = 0;
        let mut stop_loss_trades = 0;
        let mut stop_loss_fail_trades = 0;
        let mut total_return = 0.0;
        let mut max_return: f32 = -1.0;
        let mut max_loss: f32 = 0.0;
        let mut total_hold_days = 0.0;
        let mut all_returns = Vec::new();
        let mut all_trade_details = Vec::new();
        
        for result in results {
            total_trades += result.total_trades;
            winning_trades += result.winning_trades;
            losing_trades += result.losing_trades;
            stop_loss_trades += result.stop_loss_trades;
            stop_loss_fail_trades += result.stop_loss_fail_trades;
            
            total_return += result.avg_return * result.total_trades as f32;
            max_return = max_return.max(result.max_return);
            max_loss = max_loss.min(result.max_loss);
            total_hold_days += result.avg_hold_days * result.total_trades as f32;
            
            // 收集所有交易的收益率用于计算高级指标
            if let Some(details) = result.trade_details {
                for detail in &details {
                    all_returns.push(detail.return_pct);
                }
                all_trade_details.extend(details);
            }
        }
        
        let win_rate = if total_trades > 0 {
            winning_trades as f32 / total_trades as f32
        } else {
            0.0
        };
        
        let stop_loss_rate = if total_trades > 0 {
            stop_loss_trades as f32 / total_trades as f32
        } else {
            0.0
        };
        
        let stop_loss_fail_rate = if total_trades > 0 {
            stop_loss_fail_trades as f32 / total_trades as f32
        } else {
            0.0
        };
        
        let avg_return = if total_trades > 0 {
            total_return / total_trades as f32
        } else {
            0.0
        };
        
        let avg_hold_days = if total_trades > 0 {
            total_hold_days / total_trades as f32
        } else {
            0.0
        };
        
        let mut result = Self {
            total_trades,
            winning_trades,
            losing_trades,
            stop_loss_trades,
            stop_loss_fail_trades,
            win_rate,
            stop_loss_rate,
            stop_loss_fail_rate,
            avg_return,
            max_return,
            max_loss,
            avg_hold_days,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            profit_factor: 0.0,
            trade_details: if all_trade_details.is_empty() {
                None
            } else {
                Some(all_trade_details)
            },
        };
        
        // 计算高级指标
        result.calculate_advanced_metrics(&all_returns);
        
        result
    }
    
    /// 计算高级指标
    pub fn calculate_advanced_metrics(&mut self, returns: &[f32]) {
        // 计算夏普比率
        self.sharpe_ratio = Self::calculate_sharpe_ratio(returns);
        
        // 计算最大回撤
        self.max_drawdown = Self::calculate_max_drawdown(returns);
        
        // 计算盈亏比
        self.profit_factor = if self.losing_trades > 0 {
            (self.winning_trades as f32 * self.avg_return.max(0.0)) / 
            (self.losing_trades as f32 * self.max_loss.abs().max(0.001))
        } else {
            f32::INFINITY
        };
    }
    
    // 辅助方法
    fn calculate_sharpe_ratio(returns: &[f32]) -> f32 {
        if returns.is_empty() {
            return 0.0;
        }
        
        let mean: f32 = returns.iter().sum::<f32>() / returns.len() as f32;
        
        let variance = returns.iter()
            .map(|&r| (r - mean).powi(2))
            .sum::<f32>() / returns.len() as f32;
            
        let std_dev = variance.sqrt();
        
        if std_dev == 0.0 {
            return 0.0;
        }
        
        // 假设无风险利率为0
        mean / std_dev
    }
    
    fn calculate_max_drawdown(returns: &[f32]) -> f32 {
        if returns.is_empty() {
            return 0.0;
        }
        
        // 计算累积收益
        let mut cumulative = Vec::with_capacity(returns.len());
        let mut cum_return = 1.0;
        
        for &ret in returns {
            cum_return *= 1.0 + ret;
            cumulative.push(cum_return);
        }
        
        // 计算最大回撤
        let mut max_dd: f32 = 0.0;
        let mut peak = cumulative[0];
        
        for &value in &cumulative {
            if value > peak {
                peak = value;
            }
            
            let dd = (peak - value) / peak;
            max_dd = max_dd.max(dd);
        }
        
        max_dd
    }
    
    /// 格式化为人类可读的报告
    pub fn format_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("总交易次数: {}\n", self.total_trades));
        report.push_str(&format!("胜率: {:.2}%\n", self.win_rate * 100.0));
        report.push_str(&format!("止损率: {:.2}%\n", self.stop_loss_rate * 100.0));
        report.push_str(&format!("止损失败率: {:.2}%\n", self.stop_loss_fail_rate * 100.0));
        report.push_str(&format!("平均收益率: {:.2}%\n", self.avg_return * 100.0));
        report.push_str(&format!("最大收益率: {:.2}%\n", self.max_return * 100.0));
        report.push_str(&format!("最大亏损率: {:.2}%\n", self.max_loss * 100.0));
        report.push_str(&format!("平均持有天数: {:.1}天\n", self.avg_hold_days));
        report.push_str(&format!("夏普比率: {:.2}\n", self.sharpe_ratio));
        report.push_str(&format!("最大回撤: {:.2}%\n", self.max_drawdown * 100.0));
        report.push_str(&format!("盈亏比: {:.2}\n", self.profit_factor));
        
        report
    }
}
