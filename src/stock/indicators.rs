use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 计算指数移动平均线(EMA)
pub fn calculate_ema(data: &[f32], period: usize, idx: usize) -> f32 {
    if idx < period || data.len() <= idx {
        return 0.0;
    }
    
    // 计算EMA系数
    let k = 2.0 / (period as f32 + 1.0);
    
    // 初始化EMA为简单移动平均
    let mut ema = data[(idx - period + 1)..=idx].iter().sum::<f32>() / period as f32;
    
    // 计算EMA
    for i in (idx - period + 1)..=idx {
        ema = data[i] * k + ema * (1.0 - k);
    }
    
    ema
}

/// 计算移动平均线
pub fn moving_average(data: &[f32], window: usize) -> Vec<f32> {
    let len = data.len();
    let mut result = vec![0.0f32; len];
    
    for i in 0..len {
        if i + 1 < window {
            result[i] = 0.0;
        } else {
            let sum: f32 = data[i + 1 - window..=i].iter().sum();
            result[i] = sum / window as f32;
        }
    }
    
    result
}

/// 计算真实波动幅度(ATR)
pub fn calculate_atr(high: &[f32], low: &[f32], close: &[f32], window: usize) -> Vec<f32> {
    let len = high.len();
    let mut tr = vec![0.0f32; len];
    
    for i in 0..len {
        tr[i] = if i == 0 {
            high[i] - low[i]
        } else {
            let t1 = high[i] - low[i];
            let t2 = (high[i] - close[i-1]).abs();
            let t3 = (low[i] - close[i-1]).abs();
            t1.max(t2).max(t3)
        }
    }
    
    moving_average(&tr, window)
}

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

/// 计算相对强弱指标(RSI)
pub fn calculate_rsi(closes: &[f32], period: usize) -> Vec<f32> {
    let len = closes.len();
    let mut rsi = vec![0.0; len];
    
    if len <= period {
        return rsi;
    }
    
    let mut gains = vec![0.0; len];
    let mut losses = vec![0.0; len];
    
    for i in 1..len {
        let change = closes[i] - closes[i-1];
        gains[i] = if change > 0.0 { change } else { 0.0 };
        losses[i] = if change < 0.0 { -change } else { 0.0 };
    }
    
    let mut avg_gain = gains[1..=period].iter().sum::<f32>() / period as f32;
    let mut avg_loss = losses[1..=period].iter().sum::<f32>() / period as f32;
    
    rsi[period] = if avg_loss == 0.0 { 
        100.0 
    } else { 
        100.0 - (100.0 / (1.0 + avg_gain / avg_loss)) 
    };
    
    for i in (period+1)..len {
        avg_gain = (avg_gain * (period as f32 - 1.0) + gains[i]) / period as f32;
        avg_loss = (avg_loss * (period as f32 - 1.0) + losses[i]) / period as f32;
        
        rsi[i] = if avg_loss == 0.0 { 
            100.0 
        } else { 
            100.0 - (100.0 / (1.0 + avg_gain / avg_loss)) 
        };
    }
    
    rsi
}

/// 计算MACD指标
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
    
    // 初始化EMA
    fast_ema[fast_period-1] = closes[..fast_period].iter().sum::<f32>() / fast_period as f32;
    slow_ema[slow_period-1] = closes[..slow_period].iter().sum::<f32>() / slow_period as f32;
    
    // 计算EMA系数
    let fast_k = 2.0 / (fast_period as f32 + 1.0);
    let slow_k = 2.0 / (slow_period as f32 + 1.0);
    let signal_k = 2.0 / (signal_period as f32 + 1.0);
    
    // 计算快线和慢线EMA
    for i in fast_period..len {
        fast_ema[i] = closes[i] * fast_k + fast_ema[i-1] * (1.0 - fast_k);
    }
    
    for i in slow_period..len {
        slow_ema[i] = closes[i] * slow_k + slow_ema[i-1] * (1.0 - slow_k);
    }
    
    // 计算MACD线
    for i in slow_period..len {
        macd[i] = fast_ema[i] - slow_ema[i];
    }
    
    // 计算信号线
    signal[slow_period+signal_period-1] = macd[slow_period..slow_period+signal_period].iter().sum::<f32>() / signal_period as f32;
    
    for i in (slow_period+signal_period)..len {
        signal[i] = macd[i] * signal_k + signal[i-1] * (1.0 - signal_k);
    }
    
    // 计算柱状图
    for i in (slow_period+signal_period)..len {
        histogram[i] = macd[i] - signal[i];
    }
    
    (macd, signal, histogram)
}

/// 计算布林带
pub fn calculate_bollinger_bands(closes: &[f32], period: usize, std_dev_multiplier: f32) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let len = closes.len();
    let mut middle_band = vec![0.0; len];
    let mut upper_band = vec![0.0; len];
    let mut lower_band = vec![0.0; len];
    
    if len <= period {
        return (middle_band, upper_band, lower_band);
    }
    
    for i in (period-1)..len {
        let slice = &closes[(i-(period-1))..=i];
        let sma = slice.iter().sum::<f32>() / period as f32;
        middle_band[i] = sma;
        
        let std_dev = (slice.iter().map(|&x| (x - sma).powi(2)).sum::<f32>() / period as f32).sqrt();
        upper_band[i] = sma + std_dev_multiplier * std_dev;
        lower_band[i] = sma - std_dev_multiplier * std_dev;
    }
    
    (middle_band, upper_band, lower_band)
}

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
