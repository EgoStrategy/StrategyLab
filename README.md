# StrategyLab

StrategyLab 是一个用 Rust 编写的股票交易策略回测框架，旨在帮助用户开发、测试和评估各种交易策略。该框架提供了模块化的组件，包括选股策略、买入信号生成、目标设定和回测评估。

## 项目结构

```
StrategyLab/
├── src/
│   ├── backtest/       # 回测引擎
│   ├── bin/            # 可执行文件
│   ├── signals/        # 买入信号生成器
│   │   ├── price.rs    # 基于价格的信号
│   │   └── volume.rs   # 基于成交量的信号
│   ├── stock/          # 股票数据处理
│   │   ├── data_provider.rs  # 数据提供者
│   │   └── indicators.rs     # 技术指标计算
│   ├── strategies/     # 选股策略
│   │   ├── atr.rs      # 基于ATR的策略
│   │   ├── macd.rs     # 基于MACD的策略
│   │   └── rsi.rs      # 基于RSI的策略
│   ├── targets/        # 目标设定
│   │   ├── guard_target.rs   # 止损目标
│   │   └── return_target.rs  # 收益率目标
│   ├── scorecard.rs    # 策略评分卡
│   ├── lib.rs          # 库入口
│   └── main.rs         # 主程序入口
```

## 核心组件

### 1. 股票数据处理 (stock)

- **数据提供者 (data_provider.rs)**: 负责从外部数据源获取股票数据，并提供缓存和过滤功能。
- **技术指标 (indicators.rs)**: 实现了常用的技术分析指标，如EMA、ATR、RSI、MACD和布林带等。

### 2. 选股策略 (strategies)

实现了 `StockSelector` 特征的各种选股策略:

- **ATR策略 (atr.rs)**: 基于真实波动幅度(ATR)的选股策略，考虑波动性、流动性、趋势等因素。
- **MACD策略 (macd.rs)**: 基于MACD指标的选股策略。
- **RSI策略 (rsi.rs)**: 基于相对强弱指标(RSI)的选股策略。

### 3. 买入信号生成 (signals)

实现了 `BuySignalGenerator` 特征的信号生成器:

- **价格信号 (price.rs)**:
  - `ClosePriceSignal`: 基于收盘价的买入信号
  - `OpenPriceSignal`: 基于开盘价的买入信号
  - `LimitPriceSignal`: 基于限价的买入信号
- **成交量信号 (volume.rs)**:
  - `VolumeSurgeSignal`: 基于成交量突破的买入信号

### 4. 目标设定 (targets)

实现了 `Target` 特征的目标类型:

- **收益率目标 (return_target.rs)**: 在指定天数内达到目标收益率
- **止损目标 (guard_target.rs)**: 在指定天数内不触发止损

### 5. 回测引擎 (backtest)

- **BacktestEngine**: 完整的回测引擎，支持多策略、多信号、多目标的组合回测
- **Backtest**: 简化的回测类，用于单次回测

### 6. 策略评分卡 (scorecard.rs)

- **Scorecard**: 评估不同策略组合的性能，找出最佳组合

## 使用示例

### 基本回测

```rust
use strategy_lab::stock::data_provider::StockDataProvider;
use strategy_lab::strategies::atr::AtrSelector;
use strategy_lab::signals::price::OpenPriceSignal;
use strategy_lab::backtest::Backtest;
use strategy_lab::targets::return_target::ReturnTarget;

fn main() -> anyhow::Result<()> {
    // 初始化数据提供者
    let mut provider = StockDataProvider::new()?;
    
    // 获取并过滤股票
    let all_stocks = provider.get_all_stocks();
    let filtered_stocks = provider.filter_stocks(all_stocks);
    
    // 加载股票数据
    let mut stock_data = Vec::new();
    for symbol in filtered_stocks.iter().take(50) {
        if let Some(bars) = provider.get_daily_bars(symbol) {
            if bars.len() > 100 {
                stock_data.push((symbol.clone(), bars.clone()));
            }
        }
    }
    
    // 创建选股策略
    let selector = AtrSelector::default();
    
    // 运行选股策略
    let forecast_idx = 20; // 回测20天前的数据
    let candidates = selector.run(&stock_data, forecast_idx);
    
    // 生成买入信号
    let signal_generator = OpenPriceSignal;
    let signals = signal_generator.generate_signals(candidates, forecast_idx);
    
    // 设置退出目标
    let exit_target = ReturnTarget::new(0.1, 0.05, 10);
    
    // 运行回测
    let backtest = Backtest::new(exit_target);
    let results = backtest.run(signals, forecast_idx);
    
    // 打印回测结果
    println!("总交易次数: {}", results.total_trades);
    println!("胜率: {:.2}%", results.win_rate * 100.0);
    println!("平均收益率: {:.2}%", results.avg_return * 100.0);
    
    Ok(())
}
```

### 使用评分卡

```rust
use strategy_lab::strategies::{StockSelector, atr::AtrSelector};
use strategy_lab::signals::{BuySignalGenerator, price::{ClosePriceSignal, OpenPriceSignal}};
use strategy_lab::targets::{Target, return_target::ReturnTarget, guard_target::GuardTarget};
use strategy_lab::scorecard::Scorecard;

fn main() -> anyhow::Result<()> {
    // 创建选股策略
    let selectors: Vec<Box<dyn StockSelector>> = vec![
        Box::new(AtrSelector::default()),
    ];
    
    // 创建买入信号生成器
    let signals: Vec<Box<dyn BuySignalGenerator>> = vec![
        Box::new(ClosePriceSignal),
        Box::new(OpenPriceSignal),
    ];
    
    // 创建目标
    let targets: Vec<Box<dyn Target>> = vec![
        Box::new(ReturnTarget { target_return: 0.06, stop_loss: 0.05, in_days: 3 }),
        Box::new(GuardTarget { stop_loss: 0.05, in_days: 10 }),
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
```

## 技术指标

该框架实现了多种常用的技术分析指标:

- **EMA (指数移动平均线)**
- **ATR (真实波动幅度)**
- **RSI (相对强弱指标)**
- **MACD (移动平均收敛/发散指标)**
- **布林带**
- **移动平均线**
- **标准差**

## 依赖项

- **egostrategy_datahub**: 股票数据提供
- **anyhow**: 错误处理
- **rayon**: 并行计算
- **log** 和 **env_logger**: 日志记录

## 扩展

StrategyLab 设计为可扩展的框架，您可以通过实现以下特征来添加自己的组件:

- **StockSelector**: 添加新的选股策略
- **BuySignalGenerator**: 添加新的买入信号生成器
- **Target**: 添加新的目标设定

## 许可证

[MIT License](LICENSE)
