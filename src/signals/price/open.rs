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
                // 由于T+1交易制度，买入价格是forecast_idx-1天的开盘价
                let buy_price = if forecast_idx > 0 && data.len() > forecast_idx - 1 {
                    data[forecast_idx - 1].open
                } else {
                    0.0
                };
                (symbol, data, buy_price)
            })
            .filter(|(_, _, price)| *price > 0.0)
            .collect()
    }
}
