use crate::backtest::BacktestEngine;
use crate::strategies::StockSelector;
use crate::signals::BuySignalGenerator;
use crate::targets::Target;
use log::info;
use rayon::prelude::*;

/// 策略评分卡
pub struct Scorecard {
    pub back_days: usize,
    pub engine: BacktestEngine,
    pub selectors: Vec<Box<dyn StockSelector>>,
    pub signals: Vec<Box<dyn BuySignalGenerator>>,
    pub targets: Vec<Box<dyn Target>>,
}

impl Scorecard {
    /// 创建新的评分卡
    pub fn new(
        back_days: usize,
        selectors: Vec<Box<dyn StockSelector>>,
        signals: Vec<Box<dyn BuySignalGenerator>>,
        targets: Vec<Box<dyn Target>>,
    ) -> anyhow::Result<Self> {
        let mut engine = BacktestEngine::new()?;
        engine.load_data()?;
        
        Ok(Self {
            back_days,
            engine,
            selectors,
            signals,
            targets,
        })
    }
    
    /// 运行评分卡，返回所有组合的评分
    pub fn run(&self) -> Vec<Vec<Vec<f32>>> {
        info!("Running scorecard with {} selectors, {} signals, and {} targets", 
            self.selectors.len(), self.signals.len(), self.targets.len());
            
        // 返回结构: [target][selector][signal]
        self.targets.par_iter().map(|target| {
            self.selectors.par_iter().map(|selector| {
                self.signals.par_iter().map(|signal| {
                    self.engine.run_backtest(
                        selector.as_ref(),
                        signal.as_ref(),
                        target.as_ref(),
                        self.back_days
                    )
                }).collect()
            }).collect()
        }).collect()
    }
    
    /// 打印评分卡结果
    pub fn print_results(&self, results: &[Vec<Vec<f32>>]) {
        println!("\n===== 策略评分卡结果 =====\n");
        
        for (t_idx, target_results) in results.iter().enumerate() {
            println!("目标: {}", self.targets[t_idx].name());
            println!("{:-<80}", "");
            
            // 打印表头
            print!("{:<30}", "选股策略 \\ 买入信号");
            for signal in &self.signals {
                print!("{:<15}", signal.name());
            }
            println!();
            
            // 打印结果
            for (s_idx, selector_results) in target_results.iter().enumerate() {
                print!("{:<30}", self.selectors[s_idx].name());
                for &score in selector_results {
                    print!("{:<15.2}%", score * 100.0);
                }
                println!();
            }
            println!("\n");
        }
    }
    
    /// 找出最佳组合
    pub fn find_best_combination(&self, results: &[Vec<Vec<f32>>]) -> (usize, usize, usize, f32) {
        let mut best_score = 0.0;
        let mut best_target = 0;
        let mut best_selector = 0;
        let mut best_signal = 0;
        
        for (t_idx, target_results) in results.iter().enumerate() {
            for (s_idx, selector_results) in target_results.iter().enumerate() {
                for (sig_idx, &score) in selector_results.iter().enumerate() {
                    if score > best_score {
                        best_score = score;
                        best_target = t_idx;
                        best_selector = s_idx;
                        best_signal = sig_idx;
                    }
                }
            }
        }
        
        (best_target, best_selector, best_signal, best_score)
    }
    
    /// 打印最佳组合
    pub fn print_best_combination(&self, results: &[Vec<Vec<f32>>]) {
        let (best_target, best_selector, best_signal, best_score) = self.find_best_combination(results);
        
        println!("\n===== 最佳策略组合 =====\n");
        println!("目标: {}", self.targets[best_target].name());
        println!("选股策略: {}", self.selectors[best_selector].name());
        println!("买入信号: {}", self.signals[best_signal].name());
        println!("成功率: {:.2}%", best_score * 100.0);
    }
}
