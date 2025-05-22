use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 创建模拟的日线数据
pub fn create_mock_daily_bars(count: usize) -> Vec<DailyBar> {
    let mut bars = Vec::with_capacity(count);
    
    for i in 0..count {
        // 使用日期的数字表示，例如20230101
        let day = i % 30 + 1;
        let date_num = 20230100 + day;
        
        let bar = DailyBar {
            date: date_num as i32,
            open: 10.0 + (i as f32 * 0.1),
            high: 10.5 + (i as f32 * 0.1),
            low: 9.5 + (i as f32 * 0.1),
            close: 10.2 + (i as f32 * 0.1),
            volume: (10000.0 + (i as f32 * 100.0)) as i64,
            amount: (100000.0 + (i as f32 * 1000.0)) as i64,
        };
        bars.push(bar);
    }
    
    bars
}
