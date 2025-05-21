use anyhow::Result;
use egostrategy_datahub::data_provider::StockDataProvider as DataHubProvider;
use egostrategy_datahub::models::stock::{StockData as Stock, DailyData as DailyBar};
use std::collections::HashMap;

/// 股票数据提供者，封装egostrategy_datahub的功能
pub struct StockDataProvider {
    provider: DataHubProvider,
    cache: HashMap<String, Stock>,
}

impl StockDataProvider {
    /// 创建新的数据提供者实例
    pub fn new() -> Result<Self> {
        log::info!("初始化数据提供者...");
        let provider = DataHubProvider::new()?;
        log::info!("数据提供者初始化完成");
        Ok(Self {
            provider,
            cache: HashMap::new(),
        })
    }
    
    /// 获取指定股票代码的数据
    pub fn get_stock(&mut self, symbol: &str) -> Option<&Stock> {
        if !self.cache.contains_key(symbol) {
            log::debug!("缓存中没有股票 {}, 尝试获取", symbol);
            if let Some(stock) = self.provider.get_stock_by_symbol(symbol) {
                log::debug!("获取到股票 {} 的数据", symbol);
                self.cache.insert(symbol.to_string(), stock.clone());
            } else {
                log::debug!("无法获取股票 {} 的数据", symbol);
                return None;
            }
        }
        self.cache.get(symbol)
    }
    
    /// 获取所有股票列表
    pub fn get_all_stocks(&self) -> Vec<String> {
        let stocks = self.provider.get_all_stocks();
        log::info!("获取到 {} 只股票", stocks.len());
        stocks.iter().map(|stock| stock.symbol.clone()).collect()
    }
    
    /// 获取指定股票的日线数据
    pub fn get_daily_bars(&mut self, symbol: &str) -> Option<&Vec<DailyBar>> {
        let result = self.get_stock(symbol).map(|stock| &stock.daily);
        if let Some(bars) = &result {
            log::debug!("获取股票 {} 的日线数据: {} 条记录", symbol, bars.len());
        } else {
            log::debug!("获取股票 {} 的日线数据失败", symbol);
        }
        result
    }
    
    /// 获取指定股票的名称
    pub fn get_stock_name(&mut self, symbol: &str) -> Option<String> {
        self.get_stock(symbol).map(|stock| stock.name.clone())
    }
    
    /// 过滤股票列表，排除科创板、创业板等以及股价过高的股票
    pub fn filter_stocks(&mut self, symbols: Vec<String>) -> Vec<String> {
        log::info!("开始过滤股票，原始数量: {}", symbols.len());
        
        let mut filtered = Vec::new();
        let mut excluded_by_code = 0;
        let mut excluded_by_price = 0;
        
        for symbol in symbols {
            // 排除科创板(688/689)、创业板(300/301/302)等
            if symbol.starts_with("688") || 
               symbol.starts_with("689") || 
               symbol.starts_with("300") || 
               symbol.starts_with("301") || 
               symbol.starts_with("302") {
                excluded_by_code += 1;
                continue;
            }
            
            // 排除股价大于100元的股票
            if let Some(bars) = self.get_daily_bars(&symbol) {
                if let Some(last_bar) = bars.last() {
                    if last_bar.close > 100.0 {
                        excluded_by_price += 1;
                        log::debug!("过滤掉股价过高的股票: {}, 价格: {:.2}", symbol, last_bar.close);
                        continue;
                    }
                }
            }
            
            filtered.push(symbol);
        }
        
        log::info!("过滤结果: 保留 {} 只股票, 排除 {} 只科创板/创业板股票, {} 只高价股", 
            filtered.len(), excluded_by_code, excluded_by_price);
        
        filtered
    }
}
