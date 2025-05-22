/// 计算标准差
pub fn standard_deviation(data: &[f32]) -> f32 {
    if data.len() <= 1 {
        return 0.0;
    }
    
    let mean = data.iter().sum::<f32>() / data.len() as f32;
    let variance = data.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f32>() / (data.len() as f32);
    
    variance.sqrt()
}

/// 计算真实波动幅度(ATR) - 适用于倒序数据
pub fn calculate_atr(high: &[f32], low: &[f32], close: &[f32], window: usize) -> Vec<f32> {
    let len = high.len();
    let mut tr = vec![0.0f32; len];
    
    for i in 0..len {
        tr[i] = if i == len - 1 {
            high[i] - low[i]
        } else {
            let t1 = high[i] - low[i];
            let t2 = (high[i] - close[i+1]).abs();
            let t3 = (low[i] - close[i+1]).abs();
            t1.max(t2).max(t3)
        }
    }
    
    // 使用简单移动平均计算ATR
    let mut atr = vec![0.0f32; len];
    
    if len < window {
        return atr;
    }
    
    // 计算第一个ATR值
    let mut sum = 0.0;
    for i in 0..window {
        sum += tr[i];
    }
    atr[window-1] = sum / window as f32;
    
    // 计算剩余的ATR值
    for i in window..len {
        atr[i] = (atr[i-1] * (window as f32 - 1.0) + tr[i]) / window as f32;
    }
    
    atr
}

/// 计算布林带 - 适用于倒序数据
pub fn calculate_bollinger_bands(closes: &[f32], period: usize, std_dev_multiplier: f32) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let len = closes.len();
    let mut middle_band = vec![0.0; len];
    let mut upper_band = vec![0.0; len];
    let mut lower_band = vec![0.0; len];
    
    if len <= period {
        return (middle_band, upper_band, lower_band);
    }
    
    for i in 0..(len-period+1) {
        let slice = &closes[i..(i+period)];
        let sma = slice.iter().sum::<f32>() / period as f32;
        middle_band[i+period-1] = sma;
        
        let std_dev = (slice.iter().map(|&x| (x - sma).powi(2)).sum::<f32>() / period as f32).sqrt();
        upper_band[i+period-1] = sma + std_dev_multiplier * std_dev;
        lower_band[i+period-1] = sma - std_dev_multiplier * std_dev;
    }
    
    (middle_band, upper_band, lower_band)
}

/// 计算肯特纳通道(Keltner Channel)
pub fn calculate_keltner_channel(closes: &[f32], highs: &[f32], lows: &[f32], ema_period: usize, atr_period: usize, multiplier: f32) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let len = closes.len();
    let mut middle_band = vec![0.0; len];
    let mut upper_band = vec![0.0; len];
    let mut lower_band = vec![0.0; len];
    
    if len <= ema_period.max(atr_period) {
        return (middle_band, upper_band, lower_band);
    }
    
    // 计算EMA
    let k = 2.0 / (ema_period as f32 + 1.0);
    
    // 初始化EMA为简单移动平均
    let mut ema_sum = 0.0;
    for i in 0..ema_period {
        ema_sum += closes[i];
    }
    middle_band[ema_period-1] = ema_sum / ema_period as f32;
    
    // 计算剩余的EMA值
    for i in ema_period..len {
        middle_band[i] = closes[i] * k + middle_band[i-1] * (1.0 - k);
    }
    
    // 计算ATR
    let atr = calculate_atr(highs, lows, closes, atr_period);
    
    // 计算通道
    for i in atr_period.max(ema_period)..len {
        upper_band[i] = middle_band[i] + multiplier * atr[i];
        lower_band[i] = middle_band[i] - multiplier * atr[i];
    }
    
    (middle_band, upper_band, lower_band)
}
