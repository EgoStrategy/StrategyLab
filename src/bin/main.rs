use strategy_lab::stock::data_provider::StockDataProvider;
use strategy_lab::strategies::atr::AtrSelector;
use strategy_lab::strategies::StockSelector;
use strategy_lab::signals::price::OpenPriceSignal;
use strategy_lab::signals::BuySignalGenerator;
use strategy_lab::backtest::Backtest;
use strategy_lab::targets::return_target::ReturnTarget;
use strategy_lab::targets::Target;

use anyhow::Result;
use std::time::Instant;

fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();
    
    println!("初始化数据提供者...");
    let start = Instant::now();
    let mut provider = StockDataProvider::new()?;
    println!("数据提供者初始化完成，耗时: {:?}", start.elapsed());
    
    // 获取所有股票
    let all_stocks = provider.get_all_stocks();
    println!("获取到 {} 只股票", all_stocks.len());
    
    // 过滤股票
    let filtered_stocks = provider.filter_stocks(all_stocks);
    println!("过滤后剩余 {} 只股票", filtered_stocks.len());
    
    // 获取股票数据
    let mut stock_data = Vec::new();
    for symbol in filtered_stocks.iter().take(50) { // 仅处理前50只股票，加快测试速度
        if let Some(bars) = provider.get_daily_bars(symbol) {
            if bars.len() > 100 { // 确保有足够的历史数据
                stock_data.push((symbol.clone(), bars.clone()));
            }
        }
    }
    println!("加载了 {} 只股票的数据", stock_data.len());
    
    // 创建选股策略
    let selector = AtrSelector::default();
    println!("使用策略: {}", selector.name());
    
    // 运行选股策略
    let forecast_idx = 20; // 回测20天前的数据
    let candidates = selector.run(&stock_data, forecast_idx);
    println!("选出 {} 只候选股票", candidates.len());
    
    // 生成买入信号
    let signal_generator = OpenPriceSignal;
    let signals = signal_generator.generate_signals(candidates, forecast_idx);
    println!("生成 {} 个买入信号", signals.len());
    
    // 设置退出目标
    let exit_target = ReturnTarget::new(0.1, 0.05, 10);
    
    // 运行回测
    let backtest = Backtest::new(exit_target);
    let results = backtest.run(signals, forecast_idx);
    
    // 打印回测结果
    println!("\n回测结果:");
    println!("总交易次数: {}", results.total_trades);
    println!("盈利交易次数: {}", results.winning_trades);
    println!("亏损交易次数: {}", results.losing_trades);
    println!("胜率: {:.2}%", results.win_rate * 100.0);
    println!("平均收益率: {:.2}%", results.avg_return * 100.0);
    println!("最大收益率: {:.2}%", results.max_return * 100.0);
    println!("最大亏损率: {:.2}%", results.max_loss * 100.0);
    println!("平均持仓天数: {:.1} 天", results.avg_hold_days);
    
    Ok(())
}
