use crate::stock::data_provider::StockDataProvider;
use crate::strategies::StockSelector;
use crate::signals::BuySignalGenerator;
use crate::targets::Target;
use crate::backtest::result::BacktestResult;
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
        // 修改范围，从target.in_days()+1开始，确保有足够的未来数据进行评估
        // +1是因为T+1交易制度，需要额外一天用于买入
        let range: Vec<usize> = (target.in_days()+1..target.in_days()+1+back_days).collect();
        
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
        
        // 3. 评估信号 - 使用target的evaluate_signals方法
        let (total_trades, winning_trades, losing_trades, stop_loss_trades, returns, hold_days) = 
            target.evaluate_signals(signals, forecast_idx);
        
        // 4. 计算统计指标
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
            stop_loss_fail_trades: 0,
            win_rate,
            stop_loss_rate,
            stop_loss_fail_rate: 0.0,
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
        result.calculate_advanced_metrics(&returns);
        
        result
    }
}
