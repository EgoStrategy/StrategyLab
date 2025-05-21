use crate::stock::indicators::calculate_ema;
use egostrategy_datahub::models::stock::DailyData as DailyBar;
use crate::strategies::StockSelector;

/// MACD选股策略
pub struct MacdSelector {
    top_n: usize,
}

impl MacdSelector {
    /// 创建新的MACD选股策略
    pub fn new(top_n: usize) -> Self {
        Self { top_n }
    }
    
    /// 计算MACD值
    fn calculate_macd(&self, data: &[DailyBar], idx: usize) -> (f32, f32, f32) {
        if idx < 26 || data.len() <= idx {
            return (0.0, 0.0, 0.0);
        }
        
        let prices: Vec<f32> = data.iter().map(|bar| bar.close).collect();
        
        // 计算12日EMA
        let ema12 = calculate_ema(&prices, 12, idx);
        
        // 计算26日EMA
        let ema26 = calculate_ema(&prices, 26, idx);
        
        // 计算MACD值(DIF)
        let dif = ema12 - ema26;
        
        // 计算DEA(9日DIF的EMA)
        let mut dif_values = Vec::new();
        for i in (idx - 8)..=idx {
            let (diff, _, _) = self.calculate_macd(data, i);
            dif_values.push(diff);
        }
        
        let dea = dif_values.iter().sum::<f32>() / 9.0;
        
        // 计算MACD柱状图值
        let macd = 2.0 * (dif - dea);
        
        (dif, dea, macd)
    }
}

impl StockSelector for MacdSelector {
    fn name(&self) -> String {
        "MACD选股策略".to_string()
    }
    
    fn top_n(&self) -> usize {
        self.top_n
    }
    
    fn calculate_score(&self, _symbol: &str, data: &[DailyBar], forecast_idx: usize) -> f32 {
        if data.len() <= forecast_idx || forecast_idx < 26 {
            return 0.0;
        }
        
        let (_, _, macd_current) = self.calculate_macd(data, forecast_idx);
        let (_, _, macd_prev) = self.calculate_macd(data, forecast_idx - 1);
        
        // MACD由负转正，或者MACD值增长较快时，得分较高
        if macd_prev < 0.0 && macd_current > 0.0 {
            return 100.0;
        } else if macd_current > macd_prev {
            return 50.0 + (macd_current - macd_prev) * 10.0;
        } else {
            return 0.0;
        }
    }
}
