use anyhow::Result;
use egostrategy_datahub::data_provider::StockDataProvider as DataHubProvider;
use egostrategy_datahub::models::stock::{StockData as Stock, DailyData as DailyBar};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use log::{info, debug};

/// 优化的股票数据提供者
pub struct StockDataProvider {
    provider: DataHubProvider,
    cache: Arc<Mutex<HashMap<String, Stock>>>,
    name_cache: Arc<Mutex<HashMap<String, String>>>,
}

impl StockDataProvider {
    /// 创建新的数据提供者
    pub fn new() -> Result<Self> {
        info!("初始化数据提供者...");
        let provider = DataHubProvider::new()?;
        info!("数据提供者初始化完成");
        
        Ok(Self {
            provider,
            cache: Arc::new(Mutex::new(HashMap::new())),
            name_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// 获取所有股票代码
    pub fn get_all_stocks(&self) -> Vec<String> {
        let stocks = self.provider.get_all_stocks();
        info!("获取到 {} 只股票", stocks.len());
        stocks.iter().map(|stock| stock.symbol.clone()).collect()
    }
    
    /// 过滤股票
    pub fn filter_stocks(&self, symbols: Vec<String>) -> Vec<String> {
        info!("开始过滤股票，原始数量: {}", symbols.len());
        
        let mut filtered = Vec::new();
        let mut excluded_by_code = 0;
        
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
            
            filtered.push(symbol);
        }
        
        info!("过滤结果: 保留 {} 只股票, 排除 {} 只科创板/创业板股票", 
            filtered.len(), excluded_by_code);
        
        filtered
    }
    
    /// 获取股票日线数据，带缓存
    pub fn get_daily_bars(&self, symbol: &str) -> Option<Vec<DailyBar>> {
        // 先检查缓存
        {
            let cache = self.cache.lock().unwrap();
            if let Some(stock) = cache.get(symbol) {
                debug!("缓存命中: {}", symbol);
                return Some(stock.daily.clone());
            }
        }
        
        // 缓存未命中，从数据源获取
        debug!("缓存未命中: {}, 从数据源获取", symbol);
        let stock = self.fetch_stock_data(symbol)?;
        
        // 更新缓存
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(symbol.to_string(), stock.clone());
        }
        
        Some(stock.daily)
    }
    
    /// 从数据源获取股票数据
    fn fetch_stock_data(&self, symbol: &str) -> Option<Stock> {
        match self.provider.get_stock_by_symbol(symbol) {
            Some(stock) => Some(stock.clone()),
            None => {
                debug!("获取股票 {} 数据失败", symbol);
                None
            }
        }
    }
    
    /// 获取股票名称
    pub fn get_stock_name(&self, symbol: &str) -> Option<String> {
        // 先检查缓存
        {
            let cache = self.name_cache.lock().unwrap();
            if let Some(name) = cache.get(symbol) {
                return Some(name.clone());
            }
        }
        
        // 缓存未命中，从数据源获取
        let stock = self.fetch_stock_data(symbol)?;
        let name = stock.name.clone();
        
        // 更新缓存
        {
            let mut cache = self.name_cache.lock().unwrap();
            cache.insert(symbol.to_string(), name.clone());
        }
        
        Some(name)
    }
    
    /// 批量加载股票数据
    pub fn load_batch_data(&self, symbols: &[String], min_days: usize) -> Vec<(String, Vec<DailyBar>)> {
        info!("Loading data for {} stocks", symbols.len());
        
        let mut result = Vec::new();
        for symbol in symbols {
            if let Some(bars) = self.get_daily_bars(symbol) {
                if bars.len() >= min_days {
                    // 过滤掉股价过高的股票
                    if let Some(last_bar) = bars.last() {
                        if last_bar.close > 100.0 {
                            debug!("过滤掉股价过高的股票: {}, 价格: {:.2}", symbol, last_bar.close);
                            continue;
                        }
                    }
                    
                    result.push((symbol.clone(), bars));
                }
            }
        }
        
        info!("Loaded data for {} stocks", result.len());
        result
    }
}
