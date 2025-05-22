use crate::targets::Target;
use egostrategy_datahub::models::stock::DailyData as DailyBar;

/// 组合目标 - 同时满足多个目标
pub struct CombinedTarget {
    pub targets: Vec<Box<dyn Target>>,
    pub weights: Vec<f32>,
}

impl CombinedTarget {
    /// 创建新的组合目标
    pub fn new(targets: Vec<Box<dyn Target>>) -> Self {
        let count = targets.len();
        let weight = 1.0 / count as f32;
        let weights = vec![weight; count];
        
        Self { targets, weights }
    }
    
    /// 创建带权重的组合目标
    pub fn with_weights(targets: Vec<Box<dyn Target>>, weights: Vec<f32>) -> Self {
        assert_eq!(targets.len(), weights.len(), "目标数量和权重数量必须相同");
        
        // 归一化权重
        let sum: f32 = weights.iter().sum();
        let normalized_weights = weights.iter().map(|&w| w / sum).collect();
        
        Self { targets, weights: normalized_weights }
    }
}

impl Target for CombinedTarget {
    fn name(&self) -> String {
        let names: Vec<String> = self.targets.iter()
            .map(|t| t.name())
            .collect();
        
        format!("组合目标 [{}]", names.join(", "))
    }
    
    fn target_return(&self) -> f32 {
        // 使用加权平均计算目标收益率
        self.targets.iter().zip(self.weights.iter())
            .map(|(t, &w)| t.target_return() * w)
            .sum()
    }
    
    fn stop_loss(&self) -> f32 {
        // 使用最小值作为组合止损
        self.targets.iter()
            .map(|t| t.stop_loss())
            .fold(f32::MAX, |a, b| a.min(b))
    }
    
    fn in_days(&self) -> usize {
        // 使用最大值作为组合天数
        self.targets.iter()
            .map(|t| t.in_days())
            .max()
            .unwrap_or(1)
    }
    
    fn run(&self, signals: Vec<(String, Vec<DailyBar>, f32)>, forecast_idx: usize) -> f32 {
        // 对每个目标运行评估，然后计算加权平均得分
        let mut weighted_score = 0.0;
        
        for (target, &weight) in self.targets.iter().zip(self.weights.iter()) {
            // 克隆信号以便每个目标都能独立评估
            let cloned_signals = signals.iter()
                .map(|(s, d, p)| (s.clone(), d.clone(), *p))
                .collect();
                
            let score = target.run(cloned_signals, forecast_idx);
            weighted_score += score * weight;
        }
        
        weighted_score
    }
}
