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
    volume::surge::VolumeSurgeSignal,
};
use strategy_lab::targets::{
    return_target::ReturnTarget,
    guard_target::GuardTarget,
};
use strategy_lab::scorecard::Scorecard;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use serde::{Serialize, Deserialize};
use chrono::Local;
use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 配置文件路径
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,
    
    /// 回测天数
    #[arg(short, long, default_value_t = 12)]
    days: usize,
    
    /// 输出文件路径
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 运行单一策略回测
    Single {
        /// 策略名称
        #[arg(long)]
        strategy: String,
        
        /// 信号名称
        #[arg(long)]
        signal: String,
        
        /// 目标名称
        #[arg(long)]
        target: String,
    },
}

#[derive(Serialize, Deserialize)]
struct BacktestResult {
    strategy: String,
    signal: String,
    target: String,
    win_rate: f32,
    avg_return: f32,
    max_return: f32,
    max_loss: f32,
    sharpe_ratio: f32,
    max_drawdown: f32,
}

fn main() -> Result<()> {
    // 解析命令行参数
    let cli = Cli::parse();
    
    // 初始化日志
    env_logger::init();
    
    log::info!("开始运行回测...");
    // 根据命令执行不同的回测
    match &cli.command {
        Some(Commands::Single { strategy, signal, target }) => {
            // 运行单一策略回测
            run_single_backtest(strategy, signal, target, cli.days)?;
        }
        None => {
            // 运行完整评分卡
            run_full_scorecard(cli.days, cli.output)?;
        }
    }
    
    log::info!("回测完成");
    
    Ok(())
}

/// 运行单一策略回测
fn run_single_backtest(
    strategy_name: &str,
    signal_name: &str,
    target_name: &str,
    back_days: usize,
) -> Result<()> {
    log::info!("运行单一策略回测: 策略={}, 信号={}, 目标={}", strategy_name, signal_name, target_name);
    
    // 创建策略
    let selector = match strategy_name {
        "atr" => Box::new(AtrSelector {
            top_n: 10,
            lookback_days: 100,
            score_weights: Default::default(),
        }) as Box<dyn strategy_lab::strategies::StockSelector>,
        "volume_decline" => Box::new(VolumeDecliningSelector {
            top_n: 10,
            lookback_days: 30,
            min_consecutive_decline_days: 3,
            min_volume_decline_ratio: 0.1,
            price_period: 20,
            check_support_level: false,
        }) as Box<dyn strategy_lab::strategies::StockSelector>,
        "breakthrough" => Box::new(BreakthroughPullbackSelector {
            top_n: 10,
            lookback_days: 10,
            min_breakthrough_percent: 5.0,
            max_pullback_percent: 5.0,
            volume_decline_ratio: 0.7,
        }) as Box<dyn strategy_lab::strategies::StockSelector>,
        _ => return Err(anyhow::anyhow!("未知的策略: {}", strategy_name)),
    };
    
    // 创建信号
    let signal = match signal_name {
        "close" => Box::new(ClosePriceSignal) as Box<dyn strategy_lab::signals::BuySignalGenerator>,
        "open" => Box::new(OpenPriceSignal) as Box<dyn strategy_lab::signals::BuySignalGenerator>,
        "bottom_reverse" => Box::new(BottomReverseSignal::default()) as Box<dyn strategy_lab::signals::BuySignalGenerator>,
        "volume_surge" => Box::new(VolumeSurgeSignal::default()) as Box<dyn strategy_lab::signals::BuySignalGenerator>,
        _ => return Err(anyhow::anyhow!("未知的信号: {}", signal_name)),
    };
    
    // 创建目标
    let target = match target_name {
        "return_1d" => Box::new(ReturnTarget { target_return: 0.02, stop_loss: 0.01, in_days: 1 }) as Box<dyn strategy_lab::targets::Target>,
        "return_3d" => Box::new(ReturnTarget { target_return: 0.06, stop_loss: 0.01, in_days: 3 }) as Box<dyn strategy_lab::targets::Target>,
        "return_5d" => Box::new(ReturnTarget { target_return: 0.01, stop_loss: 0.01, in_days: 5 }) as Box<dyn strategy_lab::targets::Target>,
        "guard_3d" => Box::new(GuardTarget { stop_loss: 0.01, in_days: 3 }) as Box<dyn strategy_lab::targets::Target>,
        _ => return Err(anyhow::anyhow!("未知的目标: {}", target_name)),
    };
    
    // 创建评分卡
    let scorecard = Scorecard::new(
        back_days,
        vec![selector],
        vec![signal],
        vec![target],
    )?;
    
    // 运行评分卡
    let results = scorecard.run();
    
    // 打印结果
    scorecard.print_results(&results);
    
    Ok(())
}

/// 运行完整评分卡
fn run_full_scorecard(
    back_days: usize,
    output_path: Option<String>,
) -> Result<()> {
    log::info!("运行完整评分卡...");
    
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
            min_volume_decline_ratio: 0.1,
            price_period: 20,
            check_support_level: false,
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
        Box::new(VolumeSurgeSignal::default()),
    ];
    
    // 创建目标
    let targets: Vec<Box<dyn strategy_lab::targets::Target>> = vec![
        Box::new(ReturnTarget { target_return: 0.02, stop_loss: 0.01, in_days: 1 }),
        Box::new(ReturnTarget { target_return: 0.06, stop_loss: 0.01, in_days: 3 }),
        Box::new(ReturnTarget { target_return: 0.01, stop_loss: 0.01, in_days: 5 }),
        Box::new(GuardTarget { stop_loss: 0.01, in_days: 3 }),
    ];
    
    // 创建评分卡
    let scorecard = Scorecard::new(
        back_days,
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
    
    // 导出结果
    if let Some(path) = output_path {
        export_results_to_json(&scorecard, &results, best_combination, &path)?;
    }
    
    Ok(())
}

/// 导出结果到JSON
fn export_results_to_json(
    scorecard: &Scorecard,
    results: &[Vec<Vec<f32>>],
    best_combination: (usize, usize, usize, f32),
    output_path: &str,
) -> Result<()> {
    log::info!("导出结果到JSON: {}", output_path);
    
    // 创建输出目录
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }
    
    // 准备导出数据
    let mut export_data = serde_json::Map::new();
    export_data.insert("update_date".to_string(), serde_json::Value::String(Local::now().format("%Y-%m-%d").to_string()));
    
    let mut strategies = Vec::new();
    
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
                    
                    // 运行详细回测以获取性能指标
                    let backtest_result = run_detailed_backtest_for_export(
                        &scorecard.engine,
                        selector.as_ref(),
                        signal.as_ref(),
                        target.as_ref(),
                        scorecard.back_days
                    );
                    
                    // 创建策略结果
                    let mut strategy_data = serde_json::Map::new();
                    strategy_data.insert("strategy".to_string(), serde_json::Value::String(selector.name()));
                    strategy_data.insert("signal".to_string(), serde_json::Value::String(signal.name()));
                    strategy_data.insert("target".to_string(), serde_json::Value::String(target.name()));
                    strategy_data.insert("win_rate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(backtest_result.win_rate as f64).unwrap()));
                    strategy_data.insert("avg_return".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(backtest_result.avg_return as f64).unwrap()));
                    strategy_data.insert("max_return".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(backtest_result.max_return as f64).unwrap()));
                    strategy_data.insert("max_loss".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(backtest_result.max_loss as f64).unwrap()));
                    strategy_data.insert("sharpe_ratio".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(backtest_result.sharpe_ratio as f64).unwrap()));
                    strategy_data.insert("max_drawdown".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(backtest_result.max_drawdown as f64).unwrap()));
                    
                    strategies.push(serde_json::Value::Object(strategy_data));
                }
            }
        }
    }
    
    export_data.insert("strategies".to_string(), serde_json::Value::Array(strategies));
    
    // 设置最佳组合
    let (best_t, best_s, best_sig, _) = best_combination;
    let best_strategy = format!("{}-{}-{}", 
        scorecard.selectors[best_s].name(),
        scorecard.signals[best_sig].name(),
        scorecard.targets[best_t].name()
    );
    export_data.insert("best_strategy".to_string(), serde_json::Value::String(best_strategy));
    
    // 序列化为JSON
    let json = serde_json::to_string_pretty(&export_data)?;
    
    // 写入文件
    let mut file = File::create(output_path)?;
    file.write_all(json.as_bytes())?;
    
    log::info!("结果已导出到 {}", output_path);
    
    Ok(())
}

/// 运行详细回测以获取性能指标（用于导出）
fn run_detailed_backtest_for_export(
    engine: &BacktestEngine,
    selector: &dyn strategy_lab::strategies::StockSelector,
    signal: &dyn strategy_lab::signals::BuySignalGenerator,
    target: &dyn strategy_lab::targets::Target,
    back_days: usize
) -> strategy_lab::backtest::BacktestResult {
    log::info!("运行详细回测以获取性能指标...");
    
    let mut total_trades = 0;
    let mut winning_trades = 0;
    let mut losing_trades = 0;
    let mut stop_loss_trades = 0;
    let mut stop_loss_fail_trades = 0;
    let mut total_return = 0.0;
    let mut max_return: f32 = -1.0;
    let mut max_loss: f32 = 0.0;
    let mut total_hold_days = 0.0;
    let mut all_returns = Vec::new();
    
    // 对每个回测日期运行回测
    for forecast_idx in 1..=back_days {
        let result = engine.run_detailed_test(selector, signal, target, forecast_idx);
        
        // 累加结果
        total_trades += result.total_trades;
        winning_trades += result.winning_trades;
        losing_trades += result.losing_trades;
        stop_loss_trades += result.stop_loss_trades;
        stop_loss_fail_trades += result.stop_loss_fail_trades;
        total_return += result.avg_return * result.total_trades as f32;
        max_return = max_return.max(result.max_return);
        max_loss = max_loss.min(result.max_loss);
        total_hold_days += result.avg_hold_days * result.total_trades as f32;
        
        // 收集所有交易的收益率用于计算高级指标
        if let Some(details) = &result.trade_details {
            for detail in details {
                all_returns.push(detail.return_pct);
            }
        }
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
    
    let stop_loss_fail_rate = if total_trades > 0 {
        stop_loss_fail_trades as f32 / total_trades as f32
    } else {
        0.0
    };
    
    // 创建结果对象
    let mut result = strategy_lab::backtest::BacktestResult {
        total_trades,
        winning_trades,
        losing_trades,
        stop_loss_trades,
        stop_loss_fail_trades,
        win_rate,
        stop_loss_rate,
        stop_loss_fail_rate,
        avg_return,
        max_return,
        max_loss,
        avg_hold_days,
        sharpe_ratio: 0.0,
        max_drawdown: 0.0,
        profit_factor: 0.0,
        trade_details: None,
    };
    
    // 计算高级指标
    result.calculate_advanced_metrics(&all_returns);
    
    result
}
