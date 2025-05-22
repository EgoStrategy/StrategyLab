use crate::signals::BuySignalGenerator;
use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 成交量萎缩信号生成器
#[derive(Debug, Clone)]
pub struct VolumeDeclineSignal {
    pub min_consecutive_days: usize,
    pub decline_ratio: f32,
    pub price_filter: bool,
}

impl Default for VolumeDeclineSignal {
    fn default() -> Self {
        Self {
            min_consecutive_days: 3,  // 默认连续3天成交量萎缩
            decline_ratio: 0.8,       // 默认每天成交量不超过前一天的80%
            price_filter: true,       // 默认过滤价格下跌的情况
        }
    }
}

impl BuySignalGenerator for VolumeDeclineSignal {
    fn name(&self) -> String {
        "成交量萎缩信号".to_string()
    }
    
    fn generate_signals(
        &self,
        candidates: Vec<(String, Vec<DailyBar>)>,
        forecast_idx: usize,
    ) -> Vec<(String, Vec<DailyBar>, f32)> {
        candidates.into_iter()
            .filter_map(|(symbol, data)| {
                if data.len() <= forecast_idx + self.min_consecutive_days {
                    return None;
                }
                
                // 检查连续成交量萎缩
                let mut consecutive_decline = 0;
                for i in 0..self.min_consecutive_days {
                    if forecast_idx + i + 1 >= data.len() {
                        return None;
                    }
                    
                    let current_volume = data[forecast_idx + i].volume as f32;
                    let prev_volume = data[forecast_idx + i + 1].volume as f32;
                    
                    if current_volume <= prev_volume * self.decline_ratio {
                        consecutive_decline += 1;
                    } else {
                        break;
                    }
                }
                
                if consecutive_decline >= self.min_consecutive_days {
                    // 如果启用价格过滤，则检查价格是否稳定或上涨
                    if !self.price_filter || data[forecast_idx].close >= data[forecast_idx + self.min_consecutive_days].close {
                        return Some((symbol.clone(), data.clone(), data[forecast_idx].close));
                    }
                }
                
                None
            })
            .collect()
    }
}
