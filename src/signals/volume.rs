use egostrategy_datahub::models::stock::DailyData as DailyBar;
use crate::signals::BuySignalGenerator;

/// 成交量突破信号生成器
pub struct VolumeSurgeSignal {
    volume_ratio_threshold: f32,
    price_ratio_threshold: f32,
}

impl VolumeSurgeSignal {
    /// 创建新的成交量突破信号生成器
    pub fn new(volume_ratio_threshold: f32, price_ratio_threshold: f32) -> Self {
        Self {
            volume_ratio_threshold,
            price_ratio_threshold,
        }
    }
    
    /// 计算N日平均成交量
    fn calculate_avg_volume(&self, data: &[DailyBar], idx: usize, days: usize) -> f32 {
        if idx < days || data.len() <= idx {
            return 0.0;
        }
        
        let mut sum = 0.0;
        for i in (idx - days + 1)..=idx {
            sum += data[i].volume as f32;
        }
        
        sum / days as f32
    }
}

impl BuySignalGenerator for VolumeSurgeSignal {
    fn name(&self) -> String {
        "成交量突破信号".to_string()
    }
    
    fn calculate_buy_price(&self, _symbol: &str, data: &[DailyBar], forecast_idx: usize) -> f32 {
        if data.len() <= forecast_idx || forecast_idx < 5 {
            return 0.0;
        }
        
        // 计算5日平均成交量
        let avg_volume_5d = self.calculate_avg_volume(data, forecast_idx - 1, 5);
        
        // 当日成交量
        let current_volume = data[forecast_idx].volume as f32;
        
        // 计算价格变化
        let price_change_ratio = (data[forecast_idx].close - data[forecast_idx - 1].close) / data[forecast_idx - 1].close;
        
        // 成交量突破且价格上涨
        if current_volume > avg_volume_5d * self.volume_ratio_threshold && 
           price_change_ratio > self.price_ratio_threshold {
            // 返回次日开盘价作为买入价格
            return data[forecast_idx].open * 1.01; // 略高于开盘价，确保能买入
        }
        
        0.0 // 不满足条件，不生成买入信号
    }
}
