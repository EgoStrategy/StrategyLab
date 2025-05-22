use crate::stock::data_provider::StockDataProvider;
use crate::strategies::StockSelector;
use crate::signals::BuySignalGenerator;
use crate::targets::Target;
use crate::backtest::result::{BacktestResult, TradeDetail, ExitReason};
use egostrategy_datahub::models::stock::DailyData as DailyBar;
use std::sync::Arc;
use rayon::prelude::*;
use log::{info, debug};
use std::collections::HashMap;

/// 统一的回测引擎
pub struct BacktestEngine {
    data_provider: Arc<StockDataProvider>,
    stock_data: HashMap<String, Vec<DailyBar>>,
    cache_enabled: bool,
    collect_trade_details: bool,
}

impl BacktestEngine {
    /// 创建新的回测引擎
    pub fn new(cache_enabled: bool) -> anyhow::Result<Self> {
        let data_provider = Arc::new(StockDataProvider::new()?);
        Ok(Self {
            data_provider,
            stock_data: HashMap::new(),
            cache_enabled,
            collect_trade_details: false,
        })
    }
    
    /// 设置是否收集交易详情
    pub fn set_collect_trade_details(&mut self, collect: bool) {
        self.collect_trade_details = collect;
    }
    
    /// 加载股票数据
    pub fn load_data(&mut self) -> anyhow::Result<()> {
        let symbols = self.data_provider.get_all_stocks();
        let filtered_symbols = self.data_provider.filter_stocks(symbols);
        
        info!("Loading data for {} stocks", filtered_symbols.len());
        
        // 使用并行处理加速数据加载
        if self.cache_enabled {
            let stock_data: HashMap<String, Vec<DailyBar>> = filtered_symbols.par_iter()
                .filter_map(|symbol| {
                    self.data_provider.get_daily_bars(symbol)
                        .filter(|bars| bars.len() >= 120)
                        .map(|bars| (symbol.clone(), bars))
                })
                .collect();
                
            self.stock_data = stock_data;
        } else {
            for symbol in filtered_symbols {
                if let Some(daily_bars) = self.data_provider.get_daily_bars(&symbol) {
                    if daily_bars.len() >= 120 {  // 确保有足够的历史数据
                        self.stock_data.insert(symbol.clone(), daily_bars.clone());
                    }
                }
            }
        }
        
        info!("Loaded data for {} stocks", self.stock_data.len());
        Ok(())
    }
    
    /// 获取股票数据
    pub fn get_stock_data(&self) -> Vec<(String, Vec<DailyBar>)> {
        self.stock_data
            .iter()
            .map(|(symbol, data)| (symbol.clone(), data.clone()))
            .collect()
    }
    
    /// 运行单次回测
    pub fn run_single_test(
        &self,
        selector: &dyn StockSelector,
        signal_generator: &dyn BuySignalGenerator,
        target: &dyn Target,
        forecast_idx: usize,
    ) -> f32 {
        let stock_data: Vec<(String, Vec<DailyBar>)> = self.stock_data
            .iter()
            .map(|(symbol, data)| (symbol.clone(), data.clone()))
            .collect();
            
        debug!("运行单次回测: 策略={}, 信号={}, 目标={}, 预测天数={}",
            selector.name(), signal_generator.name(), target.name(), forecast_idx);
            
        // 1. 选股
        let candidates = selector.run(&stock_data, forecast_idx);
        debug!("选股结果: 选出 {} 只股票", candidates.len());
        
        // 2. 生成买入信号
        let signals = signal_generator.generate_signals(candidates, forecast_idx);
        debug!("信号生成: 生成 {} 个买入信号", signals.len());
        
        // 3. 评估目标
        let success_rate = target.run(signals, forecast_idx);
        debug!("目标评估: 成功率 = {:.2}%", success_rate * 100.0);
        
        success_rate
    }
    
    /// 运行回测
    pub fn run_backtest(
        &self,
        selector: &dyn StockSelector,
        signal_generator: &dyn BuySignalGenerator,
        target: &dyn Target,
        back_days: usize,
    ) -> f32 {
        let range: Vec<usize> = (1..=back_days).collect();
        
        let total_score: f32 = range.iter()
            .map(|&idx| {
                let forecast_idx = idx;
                self.run_single_test(selector, signal_generator, target, forecast_idx)
            })
            .sum();
            
        total_score / back_days as f32
    }
    
    /// 运行单次回测并返回详细结果
    pub fn run_detailed_test(
        &self,
        selector: &dyn StockSelector,
        signal_generator: &dyn BuySignalGenerator,
        target: &dyn Target,
        forecast_idx: usize,
    ) -> BacktestResult {
        let stock_data: Vec<(String, Vec<DailyBar>)> = self.stock_data
            .iter()
            .map(|(symbol, data)| (symbol.clone(), data.clone()))
            .collect();
            
        debug!("运行详细回测: 策略={}, 信号={}, 目标={}, 预测天数={}",
            selector.name(), signal_generator.name(), target.name(), forecast_idx);
            
        // 1. 选股
        let candidates = selector.run(&stock_data, forecast_idx);
        
        // 2. 生成买入信号
        let signals = signal_generator.generate_signals(candidates, forecast_idx);
        
        // 3. 评估信号
        self.evaluate_signals(signals, target, forecast_idx)
    }
    
    /// 评估信号
    fn evaluate_signals(
        &self,
        signals: Vec<(String, Vec<DailyBar>, f32)>,
        target: &dyn Target,
        forecast_idx: usize,
    ) -> BacktestResult {
        let mut total_trades = signals.len();
        let mut winning_trades = 0;
        let mut losing_trades = 0;
        let mut stop_loss_trades = 0;       // 触发止损的交易数
        let mut stop_loss_fail_trades = 0;  // 止损失败的交易数
        let mut returns = Vec::new();
        let mut hold_days = Vec::new();
        let mut trade_details = if self.collect_trade_details {
            Some(Vec::new())
        } else {
            None
        };
        
        for (symbol, data, buy_price) in signals.iter() {
            if buy_price <= &0.0 {
                total_trades -= 1;
                continue;
            }
            
            // 对于倒序数据，forecast_idx表示从最新数据往后数的天数
            // 我们需要检查从forecast_idx+1到forecast_idx+in_days的数据
            if data.len() <= forecast_idx + target.in_days() {
                total_trades -= 1;
                continue;
            }
            
            // 计算最大收益和止损
            let mut max_return = -1.0;
            let mut exit_day = 0;
            let mut is_win = false;
            let mut is_stop_loss = false;      // 是否触发止损
            let mut is_stop_loss_fail = false; // 是否止损失败
            let mut exit_price = 0.0;
            let mut exit_reason = ExitReason::TimeExpired;
            
            // 计算止损价
            let stop_loss_price = buy_price * (1.0 - target.stop_loss());
            
            // 检查第一个交易日是否直接低于止损价（止损失败）
            if data[forecast_idx + 1].open < stop_loss_price {
                debug!("首日止损失败: 开盘价={:.2}, 止损价={:.2}, 实际损失={:.2}%", 
                    data[forecast_idx + 1].open, stop_loss_price, 
                    (data[forecast_idx + 1].open - buy_price) / buy_price * 100.0);
                is_stop_loss_fail = true;
                max_return = (data[forecast_idx + 1].open - buy_price) / buy_price; // 实际损失
                exit_day = 1;
                exit_price = data[forecast_idx + 1].open;
                exit_reason = ExitReason::StopLossFailed;
            } else {
                // 正常交易流程
                for i in (forecast_idx + 1)..=(forecast_idx + target.in_days()) {
                    // 检查是否达到目标收益
                    let current_return = (data[i].close - buy_price) / buy_price;
                    if current_return >= target.target_return() {
                        max_return = current_return;
                        exit_day = i - forecast_idx;
                        exit_price = data[i].close;
                        is_win = true;
                        exit_reason = ExitReason::TargetReached;
                        break;
                    }
                    
                    // 检查是否跳空低开导致止损失败
                    if i > forecast_idx + 1 && data[i].open < stop_loss_price {
                        // 开盘价已低于止损价，这是止损失败
                        debug!("止损失败: 股票跳空低开, 开盘价={:.2}, 止损价={:.2}, 实际损失={:.2}%", 
                            data[i].open, stop_loss_price, (data[i].open - buy_price) / buy_price * 100.0);
                        is_stop_loss_fail = true;
                        max_return = (data[i].open - buy_price) / buy_price; // 实际损失
                        exit_day = i - forecast_idx;
                        exit_price = data[i].open;
                        exit_reason = ExitReason::StopLossFailed;
                        break;
                    }
                    
                    // 检查是否触发正常止损
                    if data[i].low <= stop_loss_price && data[i].open >= stop_loss_price {
                        // 当日最低价触及止损价，但开盘价高于止损价，这是正常止损
                        debug!("正常止损: 触发止损价, 最低价={:.2}, 止损价={:.2}, 止损比例={:.2}%", 
                            data[i].low, stop_loss_price, target.stop_loss() * 100.0);
                        is_stop_loss = true;
                        max_return = -target.stop_loss(); // 按照预设止损比例计算
                        exit_day = i - forecast_idx;
                        exit_price = stop_loss_price;
                        exit_reason = ExitReason::StopLoss;
                        break;
                    }
                    
                    // 更新最大收益
                    if current_return > max_return {
                        max_return = current_return;
                    }
                }
                
                // 如果没有提前退出，使用最后一天的收盘价计算收益
                if exit_day == 0 {
                    let last_idx = forecast_idx + target.in_days();
                    let last_return = (data[last_idx].close - buy_price) / buy_price;
                    max_return = last_return;
                    exit_day = target.in_days();
                    exit_price = data[last_idx].close;
                    exit_reason = ExitReason::TimeExpired;
                    
                    // 对于一天内目标的特殊处理
                    if target.in_days() == 1 {
                        // 检查当天是否触发止损
                        let day_idx = forecast_idx + 1;
                        if data[day_idx].low <= stop_loss_price {
                            // 当天触及止损价
                            if data[day_idx].open < stop_loss_price {
                                // 开盘就低于止损价，这是止损失败
                                debug!("一天内目标止损失败: 开盘价={:.2}, 止损价={:.2}", 
                                    data[day_idx].open, stop_loss_price);
                                is_stop_loss_fail = true;
                                max_return = (data[day_idx].open - buy_price) / buy_price;
                                exit_price = data[day_idx].open;
                                exit_reason = ExitReason::StopLossFailed;
                            } else {
                                // 开盘价高于止损价，这是正常止损
                                debug!("一天内目标正常止损: 最低价={:.2}, 止损价={:.2}", 
                                    data[day_idx].low, stop_loss_price);
                                is_stop_loss = true;
                                max_return = -target.stop_loss();
                                exit_price = stop_loss_price;
                                exit_reason = ExitReason::StopLoss;
                            }
                        }
                    }
                }
            }
            
            // 统计结果
            if is_win {
                winning_trades += 1;
            } else {
                losing_trades += 1;
                if is_stop_loss {
                    stop_loss_trades += 1;
                }
                if is_stop_loss_fail {
                    stop_loss_fail_trades += 1;
                }
            }
            
            returns.push(max_return);
            hold_days.push(exit_day as f32);
            
            // 收集交易详情
            if let Some(details) = &mut trade_details {
                // 计算日期
                let entry_date = if data.len() > forecast_idx {
                    data[forecast_idx].date.to_string()
                } else {
                    "Unknown".to_string()
                };
                
                let exit_date = if data.len() > forecast_idx + exit_day {
                    data[forecast_idx + exit_day].date.to_string()
                } else {
                    "Unknown".to_string()
                };
                
                details.push(TradeDetail {
                    symbol: symbol.clone(),
                    entry_date,
                    entry_price: *buy_price,
                    exit_date,
                    exit_price,
                    return_pct: max_return,
                    hold_days: exit_day,
                    exit_reason,
                });
            }
        }
        
        // 计算统计指标
        let win_rate = if total_trades > 0 {
            winning_trades as f32 / total_trades as f32
        } else {
            0.0
        };
        
        // 计算止损率
        let stop_loss_rate = if total_trades > 0 {
            stop_loss_trades as f32 / total_trades as f32
        } else {
            0.0
        };
        
        // 计算止损失败率
        let stop_loss_fail_rate = if total_trades > 0 {
            stop_loss_fail_trades as f32 / total_trades as f32
        } else {
            0.0
        };
        
        let avg_return = if returns.is_empty() {
            0.0
        } else {
            returns.iter().sum::<f32>() / returns.len() as f32
        };
        
        let max_return = returns.iter().fold(0.0, |max, &r| r.max(max));
        let max_loss = returns.iter().fold(0.0, |min, &r| r.min(min));
        
        let avg_hold_days = if hold_days.is_empty() {
            0.0
        } else {
            hold_days.iter().sum::<f32>() / hold_days.len() as f32
        };
        
        let mut result = BacktestResult {
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
            trade_details,
        };
        
        // 计算高级指标
        result.calculate_advanced_metrics(&returns);
        
        result
    }
}
