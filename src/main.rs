use strategy_lab::strategies::{
    StockSelector,
    atr::AtrSelector,
};
use strategy_lab::signals::{
    BuySignalGenerator,
    price::{ClosePriceSignal, OpenPriceSignal, LimitPriceSignal},
};
use strategy_lab::targets::{
    Target,
    return_target::ReturnTarget,
    guard_target::GuardTarget,
};
use strategy_lab::scorecard::Scorecard;

fn main() -> anyhow::Result<()> {
    // 初始化日志
    env_logger::init();
    
    // 创建选股策略
    let selectors: Vec<Box<dyn StockSelector>> = vec![
        Box::new(AtrSelector {
            top_n: 10,
            lookback_days: 200,
            score_weights: Default::default(),
        }),
        // 可以添加更多选股策略
    ];
    
    // 创建买入信号生成器
    let signals: Vec<Box<dyn BuySignalGenerator>> = vec![
        Box::new(ClosePriceSignal),
        Box::new(OpenPriceSignal),
        Box::new(LimitPriceSignal::default()),
    ];
    
    // 创建目标
    let targets: Vec<Box<dyn Target>> = vec![
        Box::new(ReturnTarget { target_return: 0.06, in_days: 3 }),
        Box::new(ReturnTarget { target_return: 0.03, in_days: 5 }),
        Box::new(GuardTarget { min_return: 0.01, in_days: 10 }),
    ];
    
    // 创建评分卡
    let scorecard = Scorecard::new(
        12, // 回测天数
        selectors,
        signals,
        targets,
    )?;
    
    // 运行评分卡
    let results = scorecard.run();
    
    // 打印结果
    scorecard.print_results(&results);
    
    // 打印最佳组合
    scorecard.print_best_combination(&results);
    
    Ok(())
}
