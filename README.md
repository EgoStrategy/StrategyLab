# StrategyLab

StrategyLab 是一个用 Rust 编写的股票交易策略回测框架，旨在帮助用户开发、测试和评估各种交易策略。该框架提供了模块化的组件，包括选股策略、买入信号生成、目标设定和回测评估。

## 项目结构

```
StrategyLab/
├── src/
│   ├── backtest/       # 回测引擎
│   │   ├── engine.rs   # 回测引擎实现
│   │   ├── result.rs   # 回测结果处理
│   │   └── mod.rs      # 模块导出
│   ├── bin/            # 可执行文件
│   │   ├── backtest.rs # 回测命令行工具
│   │   └── recommend.rs # 股票推荐工具
│   ├── config/         # 配置管理
│   ├── signals/        # 买入信号生成器
│   │   ├── price/      # 基于价格的信号
│   │   ├── pattern/    # 基于形态的信号
│   │   └── volume/     # 基于成交量的信号
│   ├── stock/          # 股票数据处理
│   │   ├── data_provider.rs  # 数据提供者
│   │   ├── mock_data.rs      # 模拟数据生成
│   │   └── indicators/       # 技术指标计算
│   │       ├── trend.rs      # 趋势指标
│   │       ├── oscillator.rs # 震荡指标
│   │       ├── volatility.rs # 波动指标
│   │       └── utils.rs      # 工具函数
│   ├── strategies/     # 选股策略
│   │   ├── trend/      # 趋势策略
│   │   ├── reversal/   # 反转策略
│   │   └── volume/     # 成交量策略
│   ├── targets/        # 目标设定
│   │   ├── guard_target.rs   # 止损目标
│   │   ├── return_target.rs  # 收益率目标
│   │   └── combined_target.rs # 组合目标
│   ├── utils/          # 工具函数
│   │   ├── logging.rs  # 日志工具
│   │   └── metrics.rs  # 性能指标计算
│   ├── scorecard.rs    # 策略评分卡
│   ├── lib.rs          # 库入口
│   └── main.rs         # 主程序入口
```

## 命令行使用方法

StrategyLab 提供了三个可执行文件，分别用于不同的场景：

### 1. 主程序 (strategy_lab)

主程序用于运行完整的策略评分卡，评估多种策略组合的性能。

```bash
# 运行主程序
cargo run --bin strategy_lab

# 或者编译后运行
cargo build --release
./target/release/strategy_lab
```

### 2. 回测工具 (backtest)

回测工具提供了更灵活的回测选项，支持单一策略回测和完整评分卡。

```bash
# 查看帮助
cargo run --bin backtest -- --help

# 运行完整评分卡，回测12天
cargo run --bin backtest -- --days 12

# 指定配置文件和输出路径
cargo run --bin backtest -- --config my_config.toml --output results.json

# 运行单一策略回测
cargo run --bin backtest -- single --strategy atr --signal close --target return_3d
```

可用的选项：
- `--config <FILE>`: 指定配置文件路径
- `--days <DAYS>`: 设置回测天数（默认为12）
- `--output <FILE>`: 指定输出文件路径

单一策略回测子命令选项：
- `--strategy <NAME>`: 策略名称（可选：atr, volume_decline, breakthrough）
- `--signal <NAME>`: 信号名称（可选：close, open, bottom_reverse, volume_surge）
- `--target <NAME>`: 目标名称（可选：return_1d, return_3d, return_5d, guard_3d）

### 3. 推荐工具 (recommend)

推荐工具用于生成当前市场条件下的股票推荐列表。

```bash
# 查看帮助
cargo run --bin recommend -- --help

# 使用默认参数生成推荐
cargo run --bin recommend

# 指定策略、信号和目标
cargo run --bin recommend -- -S atr -s close -t return_3d

# 指定推荐数量和输出文件
cargo run --bin recommend -- --count 10 --output my_recommendations.json
```

可用的选项：
- `-S, --strategy <NAME>`: 策略名称（默认为atr）
- `-s, --signal <NAME>`: 信号名称（默认为close）
- `-t, --target <NAME>`: 目标名称（默认为return_3d）
- `-c, --count <NUMBER>`: 推荐股票数量（默认为5）
- `-o, --output <FILE>`: 输出文件路径（默认为recommendations.json）

## 核心组件

### 1. 股票数据处理 (stock)

- **数据提供者 (data_provider.rs)**: 负责从外部数据源获取股票数据，并提供缓存和过滤功能。
- **技术指标 (indicators/)**: 实现了常用的技术分析指标，分为趋势指标、震荡指标和波动指标。

### 2. 选股策略 (strategies)

实现了 `StockSelector` 特征的各种选股策略:

- **趋势策略 (trend/)**
  - `AtrSelector`: 基于真实波动幅度(ATR)的选股策略，考虑波动性、流动性、趋势等因素。
- **反转策略 (reversal/)**
  - `BreakthroughPullbackSelector`: 突破回踩策略，寻找突破后回踩到支撑位的股票。
- **成交量策略 (volume/)**
  - `VolumeDecliningSelector`: 成交量萎缩策略，寻找成交量持续萎缩的股票。

### 3. 买入信号生成 (signals)

实现了 `BuySignalGenerator` 特征的信号生成器:

- **价格信号 (price/)**
  - `ClosePriceSignal`: 基于收盘价的买入信号
  - `OpenPriceSignal`: 基于开盘价的买入信号
- **形态信号 (pattern/)**
  - `BottomReverseSignal`: 底部反转形态信号
- **成交量信号 (volume/)**
  - `VolumeSurgeSignal`: 基于成交量突破的买入信号
  - `VolumeDeclineSignal`: 基于成交量萎缩的买入信号

### 4. 目标设定 (targets)

实现了 `Target` 特征的目标类型:

- **收益率目标 (return_target.rs)**: 在指定天数内达到目标收益率
- **止损目标 (guard_target.rs)**: 在指定天数内不触发止损
- **组合目标 (combined_target.rs)**: 同时满足多个目标

### 5. 回测引擎 (backtest)

- **BacktestEngine (engine.rs)**: 完整的回测引擎，支持多策略、多信号、多目标的组合回测
- **BacktestResult (result.rs)**: 回测结果处理，包括性能指标计算和结果合并

### 6. 策略评分卡 (scorecard.rs)

- **Scorecard**: 评估不同策略组合的性能，找出最佳组合

## 使用示例

### 基本回测

```rust
use strategy_lab::backtest::BacktestEngine;
use strategy_lab::strategies::trend::atr::AtrSelector;
use strategy_lab::signals::price::close::ClosePriceSignal;
use strategy_lab::targets::return_target::ReturnTarget;

fn main() -> anyhow::Result<()> {
    // 创建回测引擎
    let mut engine = BacktestEngine::new(true)?;
    
    // 加载股票数据
    engine.load_data()?;
    let stock_data = engine.get_stock_data();
    
    // 创建选股策略
    let selector = AtrSelector::default();
    
    // 创建买入信号生成器
    let signal = ClosePriceSignal;
    
    // 创建目标
    let target = ReturnTarget { 
        target_return: 0.06, 
        stop_loss: 0.01, 
        in_days: 3 
    };
    
    // 运行回测
    let forecast_idx = 1; // 回测1天前的数据
    let result = engine.run_detailed_test(&selector, &signal, &target, forecast_idx);
    
    // 打印回测结果
    println!("{}", result.format_report());
    
    Ok(())
}
```

### 使用评分卡

```rust
use strategy_lab::backtest::BacktestEngine;
use strategy_lab::strategies::{
    trend::atr::AtrSelector,
    volume::volume_decline::VolumeDecliningSelector,
};
use strategy_lab::signals::{
    price::close::ClosePriceSignal,
    price::open::OpenPriceSignal,
};
use strategy_lab::targets::{
    return_target::ReturnTarget,
    guard_target::GuardTarget,
};
use strategy_lab::scorecard::Scorecard;

fn main() -> anyhow::Result<()> {
    // 创建回测引擎
    let mut engine = BacktestEngine::new(true)?;
    
    // 加载股票数据
    engine.load_data()?;
    let stock_data = engine.get_stock_data();
    
    // 创建选股策略
    let selectors: Vec<Box<dyn strategy_lab::strategies::StockSelector>> = vec![
        Box::new(AtrSelector::default()),
        Box::new(VolumeDecliningSelector {
            top_n: 10,
            lookback_days: 30,
            min_consecutive_decline_days: 3,
            min_volume_decline_ratio: 0.1,
            price_period: 20,
            check_support_level: false,
        }),
    ];
    
    // 创建买入信号生成器
    let signals: Vec<Box<dyn strategy_lab::signals::BuySignalGenerator>> = vec![
        Box::new(ClosePriceSignal),
        Box::new(OpenPriceSignal),
    ];
    
    // 创建目标
    let targets: Vec<Box<dyn strategy_lab::targets::Target>> = vec![
        Box::new(ReturnTarget { target_return: 0.06, stop_loss: 0.01, in_days: 3 }),
        Box::new(GuardTarget { stop_loss: 0.01, in_days: 3 }),
    ];
    
    // 创建评分卡
    let scorecard = Scorecard::new(
        12, // 回测天数
        stock_data,
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

### 趋势指标 (trend.rs)
- **EMA (指数移动平均线)**
- **移动平均线**
- **MACD (移动平均收敛/发散指标)**

### 震荡指标 (oscillator.rs)
- **RSI (相对强弱指标)**
- **随机指标 (Stochastic Oscillator)**
- **动量指标 (Momentum)**

### 波动指标 (volatility.rs)
- **ATR (真实波动幅度)**
- **布林带 (Bollinger Bands)**
- **肯特纳通道 (Keltner Channel)**
- **标准差**

### 工具函数 (utils.rs)
- **价格数据提取**
- **涨跌幅计算**
- **累计收益率计算**
- **最大回撤计算**
- **夏普比率计算**

## 性能指标

框架提供了多种性能指标来评估策略的表现:

- **胜率 (Win Rate)**
- **平均收益率 (Average Return)**
- **最大收益 (Max Return)**
- **最大亏损 (Max Loss)**
- **夏普比率 (Sharpe Ratio)**
- **最大回撤 (Max Drawdown)**
- **盈亏比 (Profit Factor)**
- **索提诺比率 (Sortino Ratio)**
- **卡尔马比率 (Calmar Ratio)**
- **期望收益 (Expected Return)**

## 依赖项

- **egostrategy_datahub**: 股票数据提供
- **anyhow** 和 **thiserror**: 错误处理
- **rayon**: 并行计算
- **log** 和 **env_logger**: 日志记录
- **serde** 和 **serde_json**: 序列化和反序列化
- **chrono**: 日期和时间处理
- **clap**: 命令行参数解析
- **toml**: 配置文件解析

## 扩展

StrategyLab 设计为可扩展的框架，您可以通过实现以下特征来添加自己的组件:

- **StockSelector**: 添加新的选股策略
- **BuySignalGenerator**: 添加新的买入信号生成器
- **Target**: 添加新的目标设定

## 许可证

[MIT License](LICENSE)
