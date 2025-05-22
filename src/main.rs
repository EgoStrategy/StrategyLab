use strategy_lab::backtest::BacktestEngine;
use strategy_lab::strategies::{
    trend::atr::AtrSelector,
    volume::volume_decline::VolumeDecliningSelector,
    reversal::breakthrough_pullback::BreakthroughPullbackSelector,
};
use strategy_lab::signals::{
    price::close::ClosePriceSignal,
    price::open::OpenPriceSignal,
    pattern::bottom_reverse::BottomReverseSignal,
};
use strategy_lab::targets::return_target::ReturnTarget;
use strategy_lab::scorecard::Scorecard;

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use serde::{Serialize, Deserialize};
use chrono::Local;
use anyhow::Result;
use log::info;
use env_logger;

#[derive(Serialize, Deserialize)]
struct StockRecommendation {
    symbol: String,
    buy_price: f32,
    target_price: f32,
    stop_loss_price: f32,
    prev_close: Option<f32>,
}

#[derive(Serialize, Deserialize)]
struct StrategyPerformance {
    success_rate: f32,
    stop_loss_rate: f32,
    stop_loss_fail_rate: f32,
    avg_return: f32,
    max_return: f32,
    max_loss: f32,
    avg_hold_days: f32,
    sharpe_ratio: f32,
    max_drawdown: f32,
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
    best_combinations: Vec<usize>,
    strategies: Vec<StrategyResult>,
}

fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();

    // 创建选股策略
    let selectors: Vec<Box<dyn strategy_lab::strategies::StockSelector>> = vec![
        Box::new(AtrSelector {
            top_n: 10,
            lookback_days: 100,
            score_weights: Default::default(),
        }),
        Box::new(VolumeDecliningSelector {
            top_n: 10,
            lookback_days: 30,
            min_consecutive_decline_days: 3,
            min_volume_decline_ratio: 0.05,
            price_period: 20,
            check_support_level: true,
            max_support_ratio: 0.18
        }),
        Box::new(BreakthroughPullbackSelector {
            top_n: 10,
            lookback_days: 10,
            min_breakthrough_percent: 5.0,
            max_pullback_percent: 5.0,
            volume_decline_ratio: 0.7,
        }),
    ];
    
    // 创建买入信号生成器
    let signals: Vec<Box<dyn strategy_lab::signals::BuySignalGenerator>> = vec![
        Box::new(ClosePriceSignal),
        Box::new(OpenPriceSignal),
        Box::new(BottomReverseSignal::default()),
    ];
    
    // 创建目标
    let targets: Vec<Box<dyn strategy_lab::targets::Target>> = vec![
        Box::new(ReturnTarget { target_return: 0.02, stop_loss: 0.01, in_days: 1 }),
        Box::new(ReturnTarget { target_return: 0.06, stop_loss: 0.01, in_days: 3 }),
        Box::new(ReturnTarget { target_return: 0.01, stop_loss: 0.01, in_days: 5 })
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
    let best_combination = scorecard.find_best_combination(&results);
    scorecard.print_best_combination(&results);
    
    // 导出结果到JSON
    export_results_to_json(&scorecard, &results, best_combination)?;
    
    info!("评分卡运行完成");
    
    Ok(())
}

/// 导出结果到JSON
fn export_results_to_json(
    scorecard: &Scorecard,
    results: &[Vec<Vec<f32>>],
    best_combination: (usize, usize, usize, f32)
) -> Result<()> {
    info!("导出结果到JSON...");
    
    // 创建数据目录
    let data_dir = Path::new("docs/data");
    fs::create_dir_all(data_dir)?;
    
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
                        &scorecard.stock_data,
                        selector.as_ref(), 
                        signal.as_ref(), 
                        target.as_ref()
                    )?;
                    
                    // 运行详细回测以获取性能指标
                    let backtest_result = run_detailed_backtest(
                        &scorecard.engine,
                        selector.as_ref(),
                        signal.as_ref(),
                        target.as_ref(),
                        scorecard.back_days
                    );
                    
                    // 创建策略结果
                    let strategy_result = StrategyResult {
                        strategy_name: selector.name(),
                        signal_name: signal.name(),
                        target_name: target.name(),
                        performance: StrategyPerformance {
                            success_rate: score,
                            stop_loss_rate: backtest_result.stop_loss_rate,
                            stop_loss_fail_rate: backtest_result.stop_loss_fail_rate, // 添加这个字段
                            avg_return: backtest_result.avg_return,
                            max_return: backtest_result.max_return,
                            max_loss: backtest_result.max_loss,
                            avg_hold_days: backtest_result.avg_hold_days,
                            sharpe_ratio: backtest_result.sharpe_ratio,
                            max_drawdown: backtest_result.max_drawdown,
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
    
    info!("结果已导出到 docs/data/stocks.json");
    
    Ok(())
}

/// 生成推荐股票
fn generate_recommendations(
    stock_data: &[(String, Vec<egostrategy_datahub::models::stock::DailyData>)],
    selector: &dyn strategy_lab::strategies::StockSelector,
    signal: &dyn strategy_lab::signals::BuySignalGenerator,
    target: &dyn strategy_lab::targets::Target
) -> Result<Vec<StockRecommendation>> {
    info!("为策略 {} + {} 生成推荐股票...", selector.name(), signal.name());
    
    // 运行选股策略
    let forecast_idx = 0; // 使用最新数据
    let candidates = selector.run(stock_data, forecast_idx);
    
    // 生成买入信号
    let signals = signal.generate_signals(candidates, forecast_idx+1);
    
    // 创建推荐列表
    let mut recommendations = Vec::new();
    for (symbol, data, buy_price) in signals {
        if buy_price <= 0.0 {
            continue;
        }
        
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
    
    info!("生成了 {} 只推荐股票", recommendations.len());
    
    Ok(recommendations)
}

/// 运行详细回测以获取性能指标
fn run_detailed_backtest(
    engine: &BacktestEngine,
    selector: &dyn strategy_lab::strategies::StockSelector,
    signal: &dyn strategy_lab::signals::BuySignalGenerator,
    target: &dyn strategy_lab::targets::Target,
    back_days: usize
) -> strategy_lab::backtest::BacktestResult {
    info!("运行详细回测以获取性能指标...");
    
    let mut total_trades = 0;
    let mut winning_trades = 0;
    let mut losing_trades = 0;
    let mut stop_loss_trades = 0;       // 添加止损交易计数
    let mut total_return = 0.0;
    let mut max_return: f32 = -1.0;
    let mut max_loss: f32 = 0.0;
    let mut total_hold_days = 0.0;
    
    // 对每个回测日期运行回测
    for forecast_idx in 1..=back_days {
        let result = engine.run_detailed_test(selector, signal, target, forecast_idx);
        
        // 累加结果
        total_trades += result.total_trades;
        winning_trades += result.winning_trades;
        losing_trades += result.losing_trades;
        stop_loss_trades += result.stop_loss_trades;         // 累加止损交易数
        total_return += result.avg_return * result.total_trades as f32;
        max_return = max_return.max(result.max_return);
        max_loss = max_loss.min(result.max_loss);
        total_hold_days += result.avg_hold_days * result.total_trades as f32;
        
        // 记录止损和止损失败情况
        info!("回测日期 {}: 止损率={:.2}%", forecast_idx, result.stop_loss_rate * 100.0);
    }
    
    // 计算平均值
    let avg_return = if total_trades > 0 {
        total_return / total_trades as f32
    } else {
        0.0
    };
    
    let avg_hold_days = if total_trades > 0 {
        total_hold_days / total_trades as f32
    } else {
        target.in_days() as f32 / 2.0 // 默认值
    };
    
    let win_rate = if total_trades > 0 {
        winning_trades as f32 / total_trades as f32
    } else {
        0.0
    };
    
    // 计算止损率和止损失败率
    let stop_loss_rate = if total_trades > 0 {
        stop_loss_trades as f32 / total_trades as f32
    } else {
        0.0
    };
    
    // 创建结果对象
    let result = strategy_lab::backtest::BacktestResult {
        total_trades,
        winning_trades,
        losing_trades,
        stop_loss_trades,
        stop_loss_fail_trades: 0, // 添加这个字段
        win_rate,
        stop_loss_rate,
        stop_loss_fail_rate: 0.0, // 添加这个字段
        avg_return,
        max_return,
        max_loss,
        avg_hold_days,
        sharpe_ratio: 0.0,
        max_drawdown: 0.0,
        profit_factor: 0.0,
        trade_details: None, // 添加这个字段
    };
    
    result
}
