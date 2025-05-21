use crate::stock::data_provider::StockDataProvider;
use crate::strategies::StockSelector;
use crate::signals::BuySignalGenerator;
use crate::targets::Target;
use egostrategy_datahub::models::stock::DailyData as DailyBar;
use std::collections::HashMap;
use log::info;

/// 回测结果
#[derive(Debug, Clone)]
pub struct BacktestResult {
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate: f32,
    pub avg_return: f32,
    pub max_return: f32,
    pub max_loss: f32,
    pub avg_hold_days: f32,
}

/// 回测引擎
pub struct BacktestEngine {
    data_provider: StockDataProvider,
    stock_data: HashMap<String, Vec<DailyBar>>,
}

impl BacktestEngine {
    /// 创建新的回测引擎
    pub fn new() -> anyhow::Result<Self> {
        let data_provider = StockDataProvider::new()?;
        Ok(Self {
            data_provider,
            stock_data: HashMap::new(),
        })
    }
    
    /// 加载股票数据
    pub fn load_data(&mut self) -> anyhow::Result<()> {
        let symbols = self.data_provider.get_all_stocks();
        let filtered_symbols = self.data_provider.filter_stocks(symbols);
        
        info!("Loading data for {} stocks", filtered_symbols.len());
        
        for symbol in filtered_symbols {
            if let Some(daily_bars) = self.data_provider.get_daily_bars(&symbol) {
                if daily_bars.len() >= 120 {  // 确保有足够的历史数据
                    self.stock_data.insert(symbol.clone(), daily_bars.clone());
                }
            }
        }
        
        info!("Loaded data for {} stocks", self.stock_data.len());
        Ok(())
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
            
        info!("运行单次回测: 策略={}, 信号={}, 目标={}, 预测天数={}",
            selector.name(), signal_generator.name(), target.name(), forecast_idx);
            
        // 1. 选股
        let candidates = selector.run(&stock_data, forecast_idx);
        info!("选股结果: 选出 {} 只股票", candidates.len());
        
        // 2. 生成买入信号
        let signals = signal_generator.generate_signals(candidates, forecast_idx);
        info!("信号生成: 生成 {} 个买入信号", signals.len());
        
        // 3. 评估目标
        let success_rate = target.run(signals, forecast_idx);
        info!("目标评估: 成功率 = {:.2}%", success_rate * 100.0);
        
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
        let range: Vec<usize> = (1..=back_days).rev().collect();
        
        let total_score: f32 = range.iter()
            .map(|&idx| {
                let forecast_idx = idx + target.in_days();
                self.run_single_test(selector, signal_generator, target, forecast_idx)
            })
            .sum();
            
        total_score / back_days as f32
    }
}

/// 简化的回测类，用于单次回测
pub struct Backtest<T: Target> {
    target: T,
}

impl<T: Target> Backtest<T> {
    /// 创建新的回测
    pub fn new(target: T) -> Self {
        Self { target }
    }
    
    /// 运行回测
    pub fn run(&self, signals: Vec<(String, Vec<DailyBar>, f32)>, forecast_idx: usize) -> BacktestResult {
        let mut total_trades = signals.len();
        let mut winning_trades = 0;
        let mut losing_trades = 0;
        let mut returns = Vec::new();
        let mut hold_days = Vec::new();
        
        for (_, data, buy_price) in signals.iter() {
            if buy_price <= &0.0 {
                total_trades -= 1;
                continue;
            }
            
            let start_idx = data.len().saturating_sub(forecast_idx);
            let end_idx = start_idx.saturating_sub(self.target.in_days());
            
            if end_idx >= start_idx || end_idx >= data.len() {
                total_trades -= 1;
                continue;
            }
            
            // 计算最大收益和止损
            let mut max_return = -1.0;
            let mut exit_day = 0;
            let mut is_win = false;
            
            for (i, idx) in (end_idx..start_idx).enumerate() {
                let current_return = (data[idx].high - buy_price) / buy_price;
                if current_return >= self.target.target_return() {
                    max_return = current_return;
                    exit_day = i + 1;
                    is_win = true;
                    break;
                }
                
                // 检查是否触发止损
                let low_return = (data[idx].low - buy_price) / buy_price;
                if low_return <= -self.target.stop_loss() {
                    max_return = -self.target.stop_loss();
                    exit_day = i + 1;
                    break;
                }
                
                // 更新最大收益
                if current_return > max_return {
                    max_return = current_return;
                }
            }
            
            // 如果没有提前退出，使用最后一天的收盘价计算收益
            if exit_day == 0 {
                let last_idx = end_idx;
                let last_return = (data[last_idx].close - buy_price) / buy_price;
                max_return = last_return;
                exit_day = self.target.in_days();
            }
            
            // 统计结果
            if is_win {
                winning_trades += 1;
            } else {
                losing_trades += 1;
            }
            
            returns.push(max_return);
            hold_days.push(exit_day as f32);
        }
        
        // 计算统计指标
        let win_rate = if total_trades > 0 {
            winning_trades as f32 / total_trades as f32
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
        
        BacktestResult {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            avg_return,
            max_return,
            max_loss,
            avg_hold_days,
        }
    }
}
