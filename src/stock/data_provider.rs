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
        let provider = DataHubProvider::new()?;
        Ok(Self {
            provider,
            cache: HashMap::new(),
        })
    }
    
    /// 获取指定股票代码的数据
    pub fn get_stock(&mut self, symbol: &str) -> Option<&Stock> {
        if !self.cache.contains_key(symbol) {
            if let Some(stock) = self.provider.get_stock_by_symbol(symbol) {
                self.cache.insert(symbol.to_string(), stock.clone());
            } else {
                return None;
            }
        }
        self.cache.get(symbol)
    }
    
    /// 获取所有股票列表
    pub fn get_all_stocks(&self) -> Vec<String> {
        self.provider.get_all_stocks().iter().map(|stock| stock.symbol.clone()).collect()
    }
    
    /// 获取指定股票的日线数据
    pub fn get_daily_bars(&mut self, symbol: &str) -> Option<&Vec<DailyBar>> {
        self.get_stock(symbol).map(|stock| &stock.daily)
    }
    
    /// 获取指定股票的名称
    pub fn get_stock_name(&mut self, symbol: &str) -> Option<String> {
        self.get_stock(symbol).map(|stock| stock.name.clone())
    }
    
    /// 获取指定股票的交易所
    pub fn get_stock_exchange(&mut self, symbol: &str) -> Option<String> {
        self.get_stock(symbol).map(|stock| stock.exchange.clone())
    }
    
    /// 过滤股票列表，排除科创板、创业板等
    pub fn filter_stocks(&self, symbols: Vec<String>) -> Vec<String> {
        symbols.into_iter()
            .filter(|symbol| {
                // 排除科创板(688)、创业板(300/301/302)等
                !symbol.starts_with("688") && 
                !symbol.starts_with("300") && 
                !symbol.starts_with("301") && 
                !symbol.starts_with("302")
            })
            .collect()
    }
}
