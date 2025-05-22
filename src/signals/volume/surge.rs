use crate::signals::BuySignalGenerator;
use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 成交量突破信号生成器
#[derive(Debug, Clone)]
pub struct VolumeSurgeSignal {
    pub volume_ratio: f32,
    pub price_filter: bool,
}

impl Default for VolumeSurgeSignal {
    fn default() -> Self {
        Self {
            volume_ratio: 2.0,  // 默认成交量是前N日平均的2倍
            price_filter: true, // 默认过滤价格下跌的情况
        }
    }
}

impl BuySignalGenerator for VolumeSurgeSignal {
    fn name(&self) -> String {
        "成交量突破信号".to_string()
    }
    
    fn generate_signals(
        &self,
        candidates: Vec<(String, Vec<DailyBar>)>,
        forecast_idx: usize,
    ) -> Vec<(String, Vec<DailyBar>, f32)> {
        candidates.into_iter()
            .filter_map(|(symbol, data)| {
                if data.len() <= forecast_idx + 5 {  // 至少需要5天数据
                    return None;
                }
                
                let today = &data[forecast_idx];
                
                // 计算前5天的平均成交量
                let mut avg_volume = 0.0;
                for i in 1..=5 {
                    avg_volume += data[forecast_idx + i].volume as f32;
                }
                avg_volume /= 5.0;
                
                // 检查今日成交量是否突破
                let today_volume = today.volume as f32;
                if today_volume >= avg_volume * self.volume_ratio {
                    // 如果启用价格过滤，则检查价格是否上涨
                    if !self.price_filter || today.close > data[forecast_idx + 1].close {
                        return Some((symbol.clone(), data.clone(), today.close));
                    }
                }
                
                None
            })
            .collect()
    }
}
