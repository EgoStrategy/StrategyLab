use crate::signals::BuySignalGenerator;
use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 地包天买入信号
#[derive(Debug, Clone)]
pub struct BottomReverseSignal {
    pub min_body_ratio: f32,
}

impl Default for BottomReverseSignal {
    fn default() -> Self {
        Self {
            min_body_ratio: 0.5,
        }
    }
}

impl BuySignalGenerator for BottomReverseSignal {
    fn name(&self) -> String {
        "地包天信号".to_string()
    }
    
    fn generate_signals(
        &self,
        candidates: Vec<(String, Vec<DailyBar>)>,
        forecast_idx: usize,
    ) -> Vec<(String, Vec<DailyBar>, f32)> {
        candidates.into_iter()
            .filter_map(|(symbol, data)| {
                if data.len() <= forecast_idx + 1 {
                    return None;
                }
                
                let today = &data[forecast_idx];
                let yesterday = &data[forecast_idx + 1];
                
                // 检查是否形成地包天形态
                if today.open > yesterday.close && today.close < yesterday.open {
                    // 计算实体比例
                    let today_body = (today.close - today.open).abs();
                    let yesterday_body = (yesterday.close - yesterday.open).abs();
                    
                    if today_body >= yesterday_body * self.min_body_ratio {
                        return Some((symbol.clone(), data.clone(), today.close));
                    }
                }
                
                None
            })
            .collect()
    }
}
