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
use strategy_lab::stock::data_provider::StockDataProvider;

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use serde::{Serialize, Deserialize};
use chrono::Local;
use anyhow::Result;

#[derive(Serialize, Deserialize)]
struct StockRecommendation {
    symbol: String,
    name: String,
    buy_price: f32,
    target_price: f32,
    stop_loss_price: f32,
    prev_close: Option<f32>,
}

#[derive(Serialize, Deserialize)]
struct StrategyPerformance {
    success_rate: f32,
    avg_return: f32,
    max_return: f32,
    max_loss: f32,
    avg_hold_days: f32,
}

#[derive(Serialize, Deserialize)]
struct StrategyResult {
    strategy_name: String,
    signal_name: String,
    target_name: String,
    performance: StrategyPerformance,
    recommendations: Vec<StockRecommendation>,
}

#[derive(Serialize, Deserialize)]
struct ExportData {
    update_date: String,
    best_combinations: Vec<usize>,  // 索引到strategies数组
    strategies: Vec<StrategyResult>,
}

fn main() -> Result<()> {
    // 初始化日志
    std::env::set_var("RUST_LOG", "info,strategy_lab=debug");
    env_logger::init();
    
    log::info!("开始运行策略评分卡...");
    
    // 创建选股策略
    let selectors: Vec<Box<dyn StockSelector>> = vec![
        Box::new(AtrSelector {
            top_n: 10,
            lookback_days: 100,
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
        Box::new(ReturnTarget { target_return: 0.06, stop_loss: 0.02, in_days: 3 }),
        Box::new(ReturnTarget { target_return: 0.01, stop_loss: 0.02, in_days: 5 }),
        Box::new(GuardTarget { stop_loss: 0.02, in_days: 10 }),
    ];
    
    log::info!("创建评分卡...");
    
    // 创建评分卡
    let scorecard = Scorecard::new(
        12, // 回测天数
        selectors,
        signals,
        targets,
    )?;
    
    log::info!("运行评分卡...");
    
    // 运行评分卡
    let results = scorecard.run();
    
    // 打印结果
    scorecard.print_results(&results);
    
    // 打印最佳组合
    let best_combination = scorecard.find_best_combination(&results);
    scorecard.print_best_combination(&results);
    
    // 导出结果到JSON
    export_results_to_json(&scorecard, &results, best_combination)?;
    
    log::info!("评分卡运行完成");
    
    Ok(())
}

fn export_results_to_json(
    scorecard: &Scorecard,
    results: &[Vec<Vec<f32>>],
    best_combination: (usize, usize, usize, f32)
) -> Result<()> {
    log::info!("导出结果到JSON...");
    
    // 创建数据目录
    let data_dir = Path::new("docs/data");
    fs::create_dir_all(data_dir)?;
    
    // 初始化数据提供者
    let mut provider = StockDataProvider::new()?;
    
    // 准备导出数据
    let mut export_data = ExportData {
        update_date: Local::now().format("%Y-%m-%d").to_string(),
        best_combinations: vec![0, 1],  // 默认前两个为最佳组合
        strategies: Vec::new(),
    };
    
    // 获取所有策略组合的结果
    for (t_idx, target_results) in results.iter().enumerate() {
        for (s_idx, selector_results) in target_results.iter().enumerate() {
            for (sig_idx, &score) in selector_results.iter().enumerate() {
                // 只处理成功率大于0的策略
                if score > 0.0 {
                    // 获取策略、信号和目标
                    let selector = &scorecard.selectors[s_idx];
                    let signal = &scorecard.signals[sig_idx];
                    let target = &scorecard.targets[t_idx];
                    
                    // 生成推荐股票
                    let recommendations = generate_recommendations(
                        &mut provider, 
                        selector.as_ref(), 
                        signal.as_ref(), 
                        target.as_ref()
                    )?;
                    
                    // 创建策略结果
                    let strategy_result = StrategyResult {
                        strategy_name: selector.name(),
                        signal_name: signal.name(),
                        target_name: target.name(),
                        performance: StrategyPerformance {
                            success_rate: score,
                            avg_return: 0.03,  // 这些值应该从实际回测中获取
                            max_return: 0.08,
                            max_loss: -0.02,
                            avg_hold_days: target.in_days() as f32 / 2.0,
                        },
                        recommendations,
                    };
                    
                    export_data.strategies.push(strategy_result);
                }
            }
        }
    }
    
    // 设置最佳组合
    let (best_t, best_s, best_sig, _) = best_combination;
    
    // 找到最佳组合在strategies数组中的索引
    for (i, strategy) in export_data.strategies.iter().enumerate() {
        if strategy.strategy_name == scorecard.selectors[best_s].name() &&
           strategy.signal_name == scorecard.signals[best_sig].name() &&
           strategy.target_name == scorecard.targets[best_t].name() {
            export_data.best_combinations[0] = i;
            break;
        }
    }
    
    // 找到第二好的组合
    let mut second_best = (0, 0, 0, 0.0);
    for (t_idx, target_results) in results.iter().enumerate() {
        for (s_idx, selector_results) in target_results.iter().enumerate() {
            for (sig_idx, &score) in selector_results.iter().enumerate() {
                if score > second_best.3 && 
                   (t_idx != best_t || s_idx != best_s || sig_idx != best_sig) {
                    second_best = (t_idx, s_idx, sig_idx, score);
                }
            }
        }
    }
    
    // 找到第二好组合在strategies数组中的索引
    let (second_t, second_s, second_sig, _) = second_best;
    for (i, strategy) in export_data.strategies.iter().enumerate() {
        if strategy.strategy_name == scorecard.selectors[second_s].name() &&
           strategy.signal_name == scorecard.signals[second_sig].name() &&
           strategy.target_name == scorecard.targets[second_t].name() {
            export_data.best_combinations[1] = i;
            break;
        }
    }
    
    // 序列化为JSON
    let json = serde_json::to_string_pretty(&export_data)?;
    
    // 写入文件
    let file_path = data_dir.join("stocks.json");
    let mut file = File::create(file_path)?;
    file.write_all(json.as_bytes())?;
    
    log::info!("结果已导出到 docs/data/stocks.json");
    
    Ok(())
}

fn generate_recommendations(
    provider: &mut StockDataProvider,
    selector: &dyn StockSelector,
    signal: &dyn BuySignalGenerator,
    target: &dyn Target
) -> Result<Vec<StockRecommendation>> {
    log::info!("为策略 {} + {} 生成推荐股票...", selector.name(), signal.name());
    
    // 获取所有股票
    let all_stocks = provider.get_all_stocks();
    
    // 过滤股票 - 排除科创板、创业板和高价股
    let filtered_stocks = provider.filter_stocks(all_stocks);
    
    // 加载股票数据
    let mut stock_data = Vec::new();
    for symbol in filtered_stocks.iter().take(100) {
        if let Some(bars) = provider.get_daily_bars(symbol) {
            if bars.len() > 100 {
                stock_data.push((symbol.clone(), bars.clone()));
            }
        }
    }
    
    log::info!("加载了 {} 只股票的数据用于生成推荐", stock_data.len());
    
    // 运行选股策略
    let forecast_idx = 0; // 使用最新数据
    let candidates = selector.run(&stock_data, forecast_idx);
    
    // 生成买入信号
    let signals = signal.generate_signals(candidates, forecast_idx);
    
    // 创建推荐列表
    let mut recommendations = Vec::new();
    for (symbol, data, buy_price) in signals {
        if buy_price <= 0.0 {
            continue;
        }
        
        // 获取股票名称
        let name = provider.get_stock_name(&symbol).unwrap_or_else(|| symbol.clone());
        
        // 计算目标价和止损价
        let target_price = buy_price * (1.0 + target.target_return());
        let stop_loss_price = buy_price * (1.0 - target.stop_loss());
        
        // 获取前一日收盘价
        let prev_close = if data.len() > 1 {
            Some(data[data.len() - 2].close)
        } else {
            None
        };
        
        // 创建推荐
        let recommendation = StockRecommendation {
            symbol,
            name,
            buy_price,
            target_price,
            stop_loss_price,
            prev_close,
        };
        
        recommendations.push(recommendation);
    }
    
    // 限制推荐数量
    if recommendations.len() > 5 {
        recommendations.truncate(5);
    }
    
    log::info!("生成了 {} 只推荐股票", recommendations.len());
    
    Ok(recommendations)
}
