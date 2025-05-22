use crate::signals::BuySignalGenerator;
use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 开盘价信号生成器
#[derive(Debug, Clone)]
pub struct OpenPriceSignal;

impl BuySignalGenerator for OpenPriceSignal {
    fn name(&self) -> String {
        "开盘价信号".to_string()
    }
    
    fn generate_signals(
        &self,
        candidates: Vec<(String, Vec<DailyBar>)>,
        forecast_idx: usize,
    ) -> Vec<(String, Vec<DailyBar>, f32)> {
        candidates.into_iter()
            .map(|(symbol, data)| {
                let buy_price = if data.len() > forecast_idx {
                    data[forecast_idx].open
                } else {
                    0.0
                };
                (symbol, data, buy_price)
            })
            .filter(|(_, _, price)| *price > 0.0)
            .collect()
    }
}
