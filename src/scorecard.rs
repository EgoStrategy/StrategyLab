use crate::backtest::BacktestEngine;
use crate::strategies::StockSelector;
use crate::signals::BuySignalGenerator;
use crate::targets::Target;
use egostrategy_datahub::models::stock::DailyData as DailyBar;
use log::info;
use rayon::prelude::*;

/// 策略评分卡
pub struct Scorecard {
    pub back_days: usize,
    pub engine: BacktestEngine,
    pub stock_data: Vec<(String, Vec<DailyBar>)>,
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
        info!("创建评分卡...");
        let mut engine = BacktestEngine::new(true)?;
        
        // 加载股票数据
        engine.load_data()?;
        let stock_data = engine.get_stock_data();

        Ok(Self {
            back_days,
            engine,
            stock_data,
            selectors,
            signals,
            targets,
        })
    }
    
    /// 运行评分卡
    pub fn run(&self) -> Vec<Vec<Vec<f32>>> {
        info!("运行评分卡...");
        
        // 创建结果矩阵: targets x selectors x signals
        let mut results = vec![vec![vec![0.0; self.signals.len()]; self.selectors.len()]; self.targets.len()];
        
        // 使用并行处理加速评分卡运行
        let combinations: Vec<(usize, usize, usize)> = (0..self.targets.len())
            .flat_map(|t| (0..self.selectors.len())
                .flat_map(move |s| (0..self.signals.len())
                    .map(move |sig| (t, s, sig))))
            .collect();
            
        let scores: Vec<(usize, usize, usize, f32)> = combinations.par_iter()
            .map(|(t, s, sig)| {
                let target = &self.targets[*t];
                let selector = &self.selectors[*s];
                let signal = &self.signals[*sig];
                
                info!("评估组合: 策略={}, 信号={}, 目标={}",
                    selector.name(), signal.name(), target.name());
                    
                let score = self.engine.run_backtest(
                    selector.as_ref(),
                    signal.as_ref(),
                    target.as_ref(),
                    self.back_days,
                );
                
                (*t, *s, *sig, score)
            })
            .collect();
            
        // 填充结果矩阵
        for (t, s, sig, score) in scores {
            results[t][s][sig] = score;
        }
        
        results
    }
    
    /// 打印结果
    pub fn print_results(&self, results: &[Vec<Vec<f32>>]) {
        println!("评分卡结果:");
        println!("===========================================================");
        
        for (t_idx, target_results) in results.iter().enumerate() {
            let target = &self.targets[t_idx];
            println!("\n目标: {}", target.name());
            
            for (s_idx, selector_results) in target_results.iter().enumerate() {
                let selector = &self.selectors[s_idx];
                println!("  策略: {}", selector.name());
                
                for (sig_idx, &score) in selector_results.iter().enumerate() {
                    let signal = &self.signals[sig_idx];
                    println!("    信号: {}, 得分: {:.2}%", signal.name(), score * 100.0);
                }
            }
        }
        
        println!("===========================================================");
    }
    
    /// 找出最佳组合
    pub fn find_best_combination(&self, results: &[Vec<Vec<f32>>]) -> (usize, usize, usize, f32) {
        let mut best = (0, 0, 0, 0.0);
        
        for (t_idx, target_results) in results.iter().enumerate() {
            for (s_idx, selector_results) in target_results.iter().enumerate() {
                for (sig_idx, &score) in selector_results.iter().enumerate() {
                    if score > best.3 {
                        best = (t_idx, s_idx, sig_idx, score);
                    }
                }
            }
        }
        
        best
    }
    
    /// 打印最佳组合
    pub fn print_best_combination(&self, results: &[Vec<Vec<f32>>]) {
        let (t_idx, s_idx, sig_idx, score) = self.find_best_combination(results);
        
        println!("\n最佳组合:");
        println!("===========================================================");
        println!("策略: {}", self.selectors[s_idx].name());
        println!("信号: {}", self.signals[sig_idx].name());
        println!("目标: {}", self.targets[t_idx].name());
        println!("得分: {:.2}%", score * 100.0);
        println!("===========================================================");
    }
}
