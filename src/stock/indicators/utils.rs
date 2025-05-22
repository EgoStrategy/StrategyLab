use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 从DailyBar提取价格数据
pub fn extract_price_data(bars: &[DailyBar]) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    let opens: Vec<f32> = bars.iter().map(|bar| bar.open).collect();
    let highs: Vec<f32> = bars.iter().map(|bar| bar.high).collect();
    let lows: Vec<f32> = bars.iter().map(|bar| bar.low).collect();
    let closes: Vec<f32> = bars.iter().map(|bar| bar.close).collect();
    let volumes: Vec<f32> = bars.iter().map(|bar| bar.volume as f32).collect();
    let amounts: Vec<f32> = bars.iter().map(|bar| bar.amount as f32).collect();
    
    (opens, highs, lows, closes, volumes, amounts)
}

/// 计算涨跌幅
pub fn calculate_price_change(closes: &[f32]) -> Vec<f32> {
    let len = closes.len();
    let mut changes = vec![0.0; len];
    
    for i in 0..(len-1) {
        if closes[i+1] != 0.0 {
            changes[i] = (closes[i] - closes[i+1]) / closes[i+1];
        }
    }
    
    changes
}

/// 计算累计收益率
pub fn calculate_cumulative_return(changes: &[f32]) -> Vec<f32> {
    let len = changes.len();
    let mut cumulative = vec![1.0; len];
    
    for i in 1..len {
        cumulative[i] = cumulative[i-1] * (1.0 + changes[i-1]);
    }
    
    // 转换为百分比收益率
    for i in 0..len {
        cumulative[i] = cumulative[i] - 1.0;
    }
    
    cumulative
}

/// 计算最大回撤
pub fn calculate_max_drawdown(closes: &[f32]) -> f32 {
    let len = closes.len();
    if len <= 1 {
        return 0.0;
    }
    
    let mut max_price = closes[0];
    let mut max_drawdown = 0.0;
    
    for i in 1..len {
        if closes[i] > max_price {
            max_price = closes[i];
        } else {
            let drawdown = (max_price - closes[i]) / max_price;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }
    }
    
    max_drawdown
}

/// 计算夏普比率
pub fn calculate_sharpe_ratio(returns: &[f32], risk_free_rate: f32) -> f32 {
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
