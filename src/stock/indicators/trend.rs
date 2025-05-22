
/// 计算指数移动平均线(EMA) - 适用于倒序数据
pub fn calculate_ema(data: &[f32], period: usize, idx: usize) -> f32 {
    if idx >= data.len() || period > idx + 1 {
        return 0.0;
    }
    
    // 计算EMA系数
    let k = 2.0 / (period as f32 + 1.0);
    
    // 初始化EMA为简单移动平均
    let mut ema = 0.0;
    for i in 0..period {
        ema += data[idx - i];
    }
    ema /= period as f32;
    
    // 计算EMA
    for i in (0..=idx).rev().take(period) {
        ema = data[i] * k + ema * (1.0 - k);
    }
    
    ema
}

/// 计算移动平均线 - 适用于倒序数据
pub fn moving_average(data: &[f32], window: usize) -> Vec<f32> {
    let len = data.len();
    let mut result = vec![0.0f32; len];
    
    for i in 0..len {
        if i + window > len {
            result[i] = 0.0;
        } else {
            let mut sum = 0.0;
            for j in 0..window {
                sum += data[i + j];
            }
            result[i] = sum / window as f32;
        }
    }
    
    result
}

/// 计算MACD指标 - 适用于倒序数据
pub fn calculate_macd(closes: &[f32], fast_period: usize, slow_period: usize, signal_period: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let len = closes.len();
    let mut macd = vec![0.0; len];
    let mut signal = vec![0.0; len];
    let mut histogram = vec![0.0; len];
    
    if len <= slow_period {
        return (macd, signal, histogram);
    }
    
    // 计算EMA
    let mut fast_ema = vec![0.0; len];
    let mut slow_ema = vec![0.0; len];
    
    // 计算EMA系数
    let fast_k = 2.0 / (fast_period as f32 + 1.0);
    let slow_k = 2.0 / (slow_period as f32 + 1.0);
    let signal_k = 2.0 / (signal_period as f32 + 1.0);
    
    // 初始化EMA
    let mut fast_sum = 0.0;
    let mut slow_sum = 0.0;
    
    for i in 0..fast_period {
        fast_sum += closes[i];
    }
    fast_ema[fast_period-1] = fast_sum / fast_period as f32;
    
    for i in 0..slow_period {
        slow_sum += closes[i];
    }
    slow_ema[slow_period-1] = slow_sum / slow_period as f32;
    
    // 计算快线和慢线EMA
    for i in fast_period..len {
        fast_ema[i] = closes[i-fast_period] * fast_k + fast_ema[i-1] * (1.0 - fast_k);
    }
    
    for i in slow_period..len {
        slow_ema[i] = closes[i-slow_period] * slow_k + slow_ema[i-1] * (1.0 - slow_k);
    }
    
    // 计算MACD线
    for i in slow_period..len {
        macd[i] = fast_ema[i] - slow_ema[i];
    }
    
    // 计算信号线
    let mut signal_sum = 0.0;
    for i in slow_period..(slow_period+signal_period) {
        signal_sum += macd[i];
    }
    signal[slow_period+signal_period-1] = signal_sum / signal_period as f32;
    
    for i in (slow_period+signal_period)..len {
        signal[i] = macd[i-signal_period] * signal_k + signal[i-1] * (1.0 - signal_k);
    }
    
    // 计算柱状图
    for i in (slow_period+signal_period)..len {
        histogram[i] = macd[i] - signal[i];
    }
    
    (macd, signal, histogram)
}
