/// 计算相对强弱指标(RSI) - 适用于倒序数据
pub fn calculate_rsi(closes: &[f32], period: usize) -> Vec<f32> {
    let len = closes.len();
    let mut rsi = vec![0.0; len];
    
    if len <= period {
        return rsi;
    }
    
    let mut gains = vec![0.0; len];
    let mut losses = vec![0.0; len];
    
    // 注意：在倒序数据中，i-1是后一天，i是前一天
    for i in 0..(len-1) {
        let change = closes[i] - closes[i+1];
        gains[i] = if change > 0.0 { change } else { 0.0 };
        losses[i] = if change < 0.0 { -change } else { 0.0 };
    }
    
    // 计算初始平均值
    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;
    
    for i in 0..period {
        avg_gain += gains[i];
        avg_loss += losses[i];
    }
    
    avg_gain /= period as f32;
    avg_loss /= period as f32;
    
    // 第一个RSI值
    rsi[period-1] = if avg_loss == 0.0 { 
        100.0 
    } else { 
        100.0 - (100.0 / (1.0 + avg_gain / avg_loss)) 
    };
    
    // 计算剩余的RSI值
    for i in period..len {
        avg_gain = (avg_gain * (period as f32 - 1.0) + gains[i-period]) / period as f32;
        avg_loss = (avg_loss * (period as f32 - 1.0) + losses[i-period]) / period as f32;
        
        rsi[i] = if avg_loss == 0.0 { 
            100.0 
        } else { 
            100.0 - (100.0 / (1.0 + avg_gain / avg_loss)) 
        };
    }
    
    rsi
}

/// 计算随机指标(Stochastic Oscillator)
pub fn calculate_stochastic(highs: &[f32], lows: &[f32], closes: &[f32], k_period: usize, d_period: usize) -> (Vec<f32>, Vec<f32>) {
    let len = closes.len();
    let mut k_values = vec![0.0; len];
    let mut d_values = vec![0.0; len];
    
    if len <= k_period {
        return (k_values, d_values);
    }
    
    // 计算%K
    for i in k_period-1..len {
        let mut highest_high = f32::MIN;
        let mut lowest_low = f32::MAX;
        
        for j in 0..k_period {
            let idx = i - j;
            highest_high = highest_high.max(highs[idx]);
            lowest_low = lowest_low.min(lows[idx]);
        }
        
        if highest_high != lowest_low {
            k_values[i] = 100.0 * (closes[i] - lowest_low) / (highest_high - lowest_low);
        } else {
            k_values[i] = 50.0; // 如果最高价等于最低价，则取中间值
        }
    }
    
    // 计算%D (简单移动平均)
    for i in k_period+d_period-2..len {
        let mut sum = 0.0;
        for j in 0..d_period {
            sum += k_values[i-j];
        }
        d_values[i] = sum / d_period as f32;
    }
    
    (k_values, d_values)
}

/// 计算动量指标(Momentum)
pub fn calculate_momentum(closes: &[f32], period: usize) -> Vec<f32> {
    let len = closes.len();
    let mut momentum = vec![0.0; len];
    
    if len <= period {
        return momentum;
    }
    
    for i in period..len {
        momentum[i] = closes[i] - closes[i-period];
    }
    
    momentum
}
