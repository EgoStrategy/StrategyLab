/// 计算夏普比率
/// 
/// * `returns` - 收益率序列
/// * `risk_free_rate` - 无风险利率
pub fn sharpe_ratio(returns: &[f32], risk_free_rate: f32) -> f32 {
    if returns.is_empty() {
        return 0.0;
    }
    
    let mean_return = returns.iter().sum::<f32>() / returns.len() as f32;
    let excess_return = mean_return - risk_free_rate;
    
    let variance = returns.iter()
        .map(|&r| (r - mean_return).powi(2))
        .sum::<f32>() / returns.len() as f32;
    
    let std_dev = variance.sqrt();
    
    if std_dev == 0.0 {
        return 0.0;
    }
    
    excess_return / std_dev
}

/// 计算索提诺比率
/// 
/// * `returns` - 收益率序列
/// * `risk_free_rate` - 无风险利率
pub fn sortino_ratio(returns: &[f32], risk_free_rate: f32) -> f32 {
    if returns.is_empty() {
        return 0.0;
    }
    
    let mean_return = returns.iter().sum::<f32>() / returns.len() as f32;
    let excess_return = mean_return - risk_free_rate;
    
    // 只考虑负收益的标准差
    let negative_returns: Vec<f32> = returns.iter()
        .filter(|&&r| r < 0.0)
        .cloned()
        .collect();
    
    if negative_returns.is_empty() {
        return f32::INFINITY; // 没有负收益，返回无穷大
    }
    
    let downside_variance = negative_returns.iter()
        .map(|&r| r.powi(2))
        .sum::<f32>() / negative_returns.len() as f32;
    
    let downside_deviation = downside_variance.sqrt();
    
    if downside_deviation == 0.0 {
        return 0.0;
    }
    
    excess_return / downside_deviation
}

/// 计算最大回撤
/// 
/// * `values` - 资产价值序列
pub fn max_drawdown(values: &[f32]) -> f32 {
    if values.len() <= 1 {
        return 0.0;
    }
    
    let mut max_value = values[0];
    let mut max_drawdown = 0.0;
    
    for &value in values.iter().skip(1) {
        if value > max_value {
            max_value = value;
        } else {
            let drawdown = (max_value - value) / max_value;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }
    }
    
    max_drawdown
}

/// 计算卡尔马比率
/// 
/// * `returns` - 收益率序列
/// * `risk_free_rate` - 无风险利率
pub fn calmar_ratio(returns: &[f32], values: &[f32], risk_free_rate: f32) -> f32 {
    if returns.is_empty() || values.len() <= 1 {
        return 0.0;
    }
    
    let mean_return = returns.iter().sum::<f32>() / returns.len() as f32;
    let excess_return = mean_return - risk_free_rate;
    
    let mdd = max_drawdown(values);
    
    if mdd == 0.0 {
        return f32::INFINITY; // 没有回撤，返回无穷大
    }
    
    excess_return / mdd
}

/// 计算胜率
/// 
/// * `returns` - 收益率序列
pub fn win_rate(returns: &[f32]) -> f32 {
    if returns.is_empty() {
        return 0.0;
    }
    
    let winning_trades = returns.iter().filter(|&&r| r > 0.0).count();
    winning_trades as f32 / returns.len() as f32
}

/// 计算盈亏比
/// 
/// * `returns` - 收益率序列
pub fn profit_factor(returns: &[f32]) -> f32 {
    let profits: f32 = returns.iter().filter(|&&r| r > 0.0).sum();
    let losses: f32 = returns.iter().filter(|&&r| r < 0.0).map(|&r| r.abs()).sum();
    
    if losses == 0.0 {
        return f32::INFINITY; // 没有亏损，返回无穷大
    }
    
    profits / losses
}

/// 计算期望收益
/// 
/// * `returns` - 收益率序列
pub fn expected_return(returns: &[f32]) -> f32 {
    if returns.is_empty() {
        return 0.0;
    }
    
    let win_rate = win_rate(returns);
    let avg_win = returns.iter().filter(|&&r| r > 0.0).sum::<f32>() / 
                 returns.iter().filter(|&&r| r > 0.0).count().max(1) as f32;
    let avg_loss = returns.iter().filter(|&&r| r < 0.0).sum::<f32>() / 
                  returns.iter().filter(|&&r| r < 0.0).count().max(1) as f32;
    
    win_rate * avg_win + (1.0 - win_rate) * avg_loss
}
