// StrategyLab 股票荐股工具脚本

// 全局变量存储策略数据
let strategyData = null;

document.addEventListener('DOMContentLoaded', function() {
    // 加载策略数据
    loadStrategyData();
});

// 加载策略数据
async function loadStrategyData() {
    try {
        const response = await fetch('data/stocks.json');
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        strategyData = await response.json();
        
        // 更新界面
        updateUI();
    } catch (error) {
        console.error('加载策略数据失败:', error);
        document.getElementById('strategy-cards').innerHTML = `
            <div class="col-12 text-center py-5">
                <div class="alert alert-danger">
                    <i class="bi bi-exclamation-triangle-fill"></i> 
                    加载策略数据失败，请稍后再试。
                </div>
            </div>
        `;
    }
}

// 更新界面
function updateUI() {
    if (!strategyData) return;
    
    // 更新日期
    document.getElementById('update-date').textContent = strategyData.update_date;
    
    // 清除加载提示
    document.getElementById('strategy-cards').innerHTML = '';
    
    // 添加最佳策略卡片
    addBestStrategyCards();
    
    // 更新性能表格
    updatePerformanceTable();
    
    // 初始化图表
    initPerformanceChart();
}

// 添加最佳策略卡片
function addBestStrategyCards() {
    const cardsContainer = document.getElementById('strategy-cards');
    
    // 获取最佳策略组合
    const bestStrategies = strategyData.best_combinations.map(index => strategyData.strategies[index]);
    
    bestStrategies.forEach((strategy, index) => {
        const cardHtml = createStrategyCard(strategy, index + 1);
        
        const colDiv = document.createElement('div');
        colDiv.className = 'col-md-6';
        colDiv.innerHTML = cardHtml;
        
        cardsContainer.appendChild(colDiv);
    });
}

// 创建策略卡片HTML
function createStrategyCard(strategy, rank) {
    const iconClass = rank === 1 ? 'bi-trophy' : 'bi-award';
    const performance = strategy.performance;
    
    let stocksHtml = '';
    
    // 根据信号类型决定表格列
    const isLimitPrice = strategy.signal_name.includes('限价');
    
    // 表头
    stocksHtml += `
        <table class="table table-sm table-hover">
            <thead>
                <tr>
                    <th>代码</th>
                    <th>名称</th>
                    ${isLimitPrice ? '<th>前收盘</th>' : ''}
                    <th>买入价</th>
                    <th>目标价</th>
                    <th>止损价</th>
                </tr>
            </thead>
            <tbody>
    `;
    
    // 表格内容
    strategy.recommendations.forEach(stock => {
        stocksHtml += `
            <tr>
                <td>${stock.symbol}</td>
                <td>${stock.name}</td>
                ${isLimitPrice ? `<td>${stock.prev_close?.toFixed(2) || '-'}</td>` : ''}
                <td>${stock.buy_price.toFixed(2)}</td>
                <td>${stock.target_price.toFixed(2)}</td>
                <td>${stock.stop_loss_price.toFixed(2)}</td>
            </tr>
        `;
    });
    
    stocksHtml += `
            </tbody>
        </table>
    `;
    
    // 构建完整卡片
    return `
        <div class="card strategy-card">
            <div class="card-header">
                <h5><i class="bi ${iconClass}"></i> 最佳策略组合 #${rank}</h5>
            </div>
            <div class="card-body">
                <h5 class="card-title">${strategy.strategy_name} + ${strategy.signal_name}</h5>
                <p class="card-text">目标: ${strategy.target_name}</p>
                
                <div class="row mb-3">
                    <div class="col-6">
                        <div class="d-flex justify-content-between">
                            <span>成功率:</span>
                            <span class="fw-bold">${(performance.success_rate * 100).toFixed(1)}%</span>
                        </div>
                        <div class="progress">
                            <div class="progress-bar bg-success" role="progressbar" 
                                style="width: ${performance.success_rate * 100}%" 
                                aria-valuenow="${performance.success_rate * 100}" 
                                aria-valuemin="0" aria-valuemax="100"></div>
                        </div>
                    </div>
                    <div class="col-6">
                        <div class="d-flex justify-content-between">
                            <span>止损率:</span>
                            <span class="fw-bold">${(performance.stop_loss_rate * 100).toFixed(1)}%</span>
                        </div>
                        <div class="progress">
                            <div class="progress-bar bg-warning" role="progressbar" 
                                style="width: ${performance.stop_loss_rate * 100}%" 
                                aria-valuenow="${performance.stop_loss_rate * 100}" 
                                aria-valuemin="0" aria-valuemax="100"></div>
                        </div>
                    </div>
                </div>
                
                <div class="row mb-3">
                    <div class="col-6">
                        <div class="d-flex justify-content-between">
                            <span>止损失败率:</span>
                            <span class="fw-bold">${(performance.stop_loss_fail_rate * 100).toFixed(1)}%</span>
                        </div>
                        <div class="progress">
                            <div class="progress-bar bg-danger" role="progressbar" 
                                style="width: ${performance.stop_loss_fail_rate * 100}%" 
                                aria-valuenow="${performance.stop_loss_fail_rate * 100}" 
                                aria-valuemin="0" aria-valuemax="100"></div>
                        </div>
                    </div>
                    <div class="col-6">
                        <div class="d-flex justify-content-between">
                            <span>平均收益:</span>
                            <span class="fw-bold">${(performance.avg_return * 100).toFixed(1)}%</span>
                        </div>
                        <div class="progress">
                            <div class="progress-bar bg-info" role="progressbar" 
                                style="width: ${performance.avg_return * 1000}%" 
                                aria-valuenow="${performance.avg_return * 100}" 
                                aria-valuemin="0" aria-valuemax="100"></div>
                        </div>
                    </div>
                </div>
                
                <h6 class="mt-4 mb-3">策略说明</h6>
                <ul>
                    <li>选股: ${strategy.strategy_name}</li>
                    <li>买入: ${strategy.signal_name}</li>
                    <li>卖出: ${getTargetDescription(strategy.target_name)}</li>
                    <li>持仓: 平均 ${performance.avg_hold_days.toFixed(1)} 天</li>
                </ul>
                
                <h6 class="mt-4 mb-3">今日推荐股票</h6>
                <div class="stock-list">
                    ${stocksHtml}
                </div>
            </div>
        </div>
    `;
}

// 获取目标描述
function getTargetDescription(targetName) {
    if (targetName.includes('收益率')) {
        const match = targetName.match(/(\d+)天内收益率达到(\d+)%/);
        if (match) {
            return `达到${match[2]}%收益目标或触发止损`;
        }
    } else if (targetName.includes('止损')) {
        const match = targetName.match(/(\d+)天内不触发(\d+)%止损/);
        if (match) {
            return `${match[1]}天内不触发${match[2]}%止损`;
        }
    }
    return targetName;
}

// 更新性能表格
function updatePerformanceTable() {
    // 准备表格数据
    const tableData = [];
    
    strategyData.strategies.forEach((strategy, index) => {
        const perf = strategy.performance;
        const isBest = strategyData.best_combinations.includes(index);
        
        // 存储策略索引，用于获取详细信息
        tableData.push({
            strategyIndex: index,
            name: `${strategy.strategy_name} + ${strategy.signal_name} (${getShortTargetName(strategy.target_name)})`,
            successRate: (perf.success_rate * 100).toFixed(1) + '%',
            stopLossRate: (perf.stop_loss_rate * 100).toFixed(1) + '%',
            stopLossFailRate: (perf.stop_loss_fail_rate * 100).toFixed(1) + '%',  // 新增：止损失败率
            avgReturn: (perf.avg_return * 100).toFixed(1) + '%',
            maxReturn: (perf.max_return * 100).toFixed(1) + '%',
            maxLoss: (perf.max_loss * 100).toFixed(1) + '%',
            avgHoldDays: perf.avg_hold_days.toFixed(1),
            isBest: isBest
        });
    });
    
    // 销毁现有表格实例（如果存在）
    if ($.fn.DataTable.isDataTable('#performance-table')) {
        $('#performance-table').DataTable().destroy();
    }
    
    // 初始化表格
    const table = $('#performance-table').DataTable({
        data: tableData,
        columns: [
            {
                className: 'dt-control',
                orderable: false,
                data: null,
                defaultContent: '',
                width: '30px'
            },
            { 
                title: '策略组合', 
                data: 'name',
                render: function(data, type, row) {
                    if (row.isBest) {
                        return `<span class="badge bg-primary me-2">推荐</span>${data}`;
                    }
                    return data;
                }
            },
            { title: '成功率', data: 'successRate' },
            { title: '止损率', data: 'stopLossRate' },
            { title: '止损失败率', data: 'stopLossFailRate' },  // 新增：止损失败率列
            { title: '平均收益', data: 'avgReturn' },
            { title: '最大收益', data: 'maxReturn' },
            { title: '最大亏损', data: 'maxLoss' },
            { title: '平均持仓天数', data: 'avgHoldDays' }
        ],
        order: [[2, 'desc']], // 默认按成功率降序排列
        language: {
            "sProcessing": "处理中...",
            "sLengthMenu": "显示 _MENU_ 项结果",
            "sZeroRecords": "没有匹配结果",
            "sInfo": "显示第 _START_ 至 _END_ 项结果，共 _TOTAL_ 项",
            "sInfoEmpty": "显示第 0 至 0 项结果，共 0 项",
            "sInfoFiltered": "(由 _MAX_ 项结果过滤)",
            "sInfoPostFix": "",
            "sSearch": "搜索:",
            "sUrl": "",
            "sEmptyTable": "表中数据为空",
            "sLoadingRecords": "载入中...",
            "sInfoThousands": ",",
            "oPaginate": {
                "sFirst": "首页",
                "sPrevious": "上页",
                "sNext": "下页",
                "sLast": "末页"
            },
            "oAria": {
                "sSortAscending": ": 以升序排列此列",
                "sSortDescending": ": 以降序排列此列"
            }
        },
        createdRow: function(row, data) {
            // 为最佳策略添加高亮样式
            if (data.isBest) {
                $(row).addClass('table-primary');
            }
        },
        responsive: true,
        paging: false,
        info: false
    });
    
    // 添加行点击事件，展开/折叠详情
    $('#performance-table tbody').on('click', 'td.dt-control', function() {
        const tr = $(this).closest('tr');
        const row = table.row(tr);
        
        if (row.child.isShown()) {
            // 已经展开，需要折叠
            row.child.hide();
            tr.removeClass('shown');
        } else {
            // 需要展开
            const rowData = row.data();
            const strategy = strategyData.strategies[rowData.strategyIndex];
            row.child(formatStrategyDetails(strategy)).show();
            tr.addClass('shown');
        }
    });
}

// 获取简短的目标名称
function getShortTargetName(targetName) {
    if (targetName.includes('收益率')) {
        const match = targetName.match(/(\d+)天(\d+)%/);
        if (match) {
            return `${match[1]}天${match[2]}%`;
        }
    } else if (targetName.includes('止损')) {
        const match = targetName.match(/(\d+)天.*?(\d+)%/);
        if (match) {
            return `${match[1]}天不止损`;
        }
    }
    return targetName;
}

// 初始化性能对比图表
function initPerformanceChart() {
    const chartContainer = document.getElementById('performance-chart');
    chartContainer.innerHTML = ''; // 清空容器
    
    // 创建第一个图表的容器
    const chartItem1 = document.createElement('div');
    chartItem1.className = 'chart-item';
    chartContainer.appendChild(chartItem1);
    
    // 创建第一个图表的画布
    const canvas1 = document.createElement('canvas');
    canvas1.id = 'success-return-chart';
    chartItem1.appendChild(canvas1);
    
    // 准备图表数据
    const labels = strategyData.strategies.map(s => {
        const strategyName = s.strategy_name.replace('策略', '');
        const signalName = s.signal_name.replace('次日', '').replace('买入', '');
        const targetShort = getShortTargetName(s.target_name);
        return `${strategyName}+${signalName}(${targetShort})`;
    });
    
    const successRates = strategyData.strategies.map(s => s.performance.success_rate * 100);
    const avgReturns = strategyData.strategies.map(s => s.performance.avg_return * 100);
    
    // 高亮最佳策略
    const backgroundColors1 = strategyData.strategies.map((s, i) => {
        return strategyData.best_combinations.includes(i) 
            ? 'rgba(40, 167, 69, 0.7)' 
            : 'rgba(108, 117, 125, 0.7)';
    });
    
    const backgroundColors2 = strategyData.strategies.map((s, i) => {
        return strategyData.best_combinations.includes(i) 
            ? 'rgba(0, 123, 255, 0.7)' 
            : 'rgba(108, 117, 125, 0.4)';
    });
    
    new Chart(canvas1, {
        type: 'bar',
        data: {
            labels: labels,
            datasets: [
                {
                    label: '成功率',
                    data: successRates,
                    backgroundColor: backgroundColors1,
                    borderColor: backgroundColors1.map(c => c.replace('0.7', '1')),
                    borderWidth: 1,
                    yAxisID: 'y'
                },
                {
                    label: '平均收益率',
                    data: avgReturns,
                    backgroundColor: backgroundColors2,
                    borderColor: backgroundColors2.map(c => c.replace('0.7', '1')),
                    borderWidth: 1,
                    yAxisID: 'y1'
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            aspectRatio: 2, // 设置宽高比
            scales: {
                y: {
                    beginAtZero: true,
                    title: {
                        display: true,
                        text: '成功率 (%)'
                    },
                    position: 'left'
                },
                y1: {
                    beginAtZero: true,
                    title: {
                        display: true,
                        text: '收益率 (%)'
                    },
                    position: 'right',
                    grid: {
                        drawOnChartArea: false
                    }
                }
            },
            plugins: {
                title: {
                    display: true,
                    text: '策略成功率与收益率对比'
                },
                legend: {
                    position: 'top'
                }
            }
        }
    });
    
    // 创建第二个图表的容器
    const chartItem2 = document.createElement('div');
    chartItem2.className = 'chart-item';
    chartContainer.appendChild(chartItem2);
    
    // 创建第二个图表 - 收益率与持仓天数散点图
    const canvas2 = document.createElement('canvas');
    canvas2.id = 'holding-return-chart';
    chartItem2.appendChild(canvas2);
    
    // 准备散点图数据
    const scatterData = strategyData.strategies.map((s, i) => {
        return {
            x: s.performance.avg_hold_days,
            y: s.performance.avg_return * 100,
            r: s.performance.success_rate * 20, // 气泡大小基于成功率
            label: labels[i]
        };
    });
    
    const datasets = strategyData.strategies.map((s, i) => {
        const isHighlighted = strategyData.best_combinations.includes(i);
        const color = isHighlighted 
            ? (i === strategyData.best_combinations[0] ? 'rgba(40, 167, 69, 0.7)' : 'rgba(0, 123, 255, 0.7)') 
            : 'rgba(108, 117, 125, 0.5)';
        
        return {
            label: labels[i],
            data: [scatterData[i]],
            backgroundColor: color,
            borderColor: color.replace('0.7', '1').replace('0.5', '0.8'),
            borderWidth: isHighlighted ? 2 : 1,
            pointRadius: scatterData[i].r,
            pointHoverRadius: scatterData[i].r + 2
        };
    });
    
    new Chart(canvas2, {
        type: 'bubble',
        data: {
            datasets: datasets
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            aspectRatio: 2, // 设置宽高比
            scales: {
                x: {
                    title: {
                        display: true,
                        text: '平均持仓天数'
                    }
                },
                y: {
                    title: {
                        display: true,
                        text: '平均收益率 (%)'
                    }
                }
            },
            plugins: {
                title: {
                    display: true,
                    text: '收益率与持仓天数对比 (气泡大小表示成功率)'
                },
                legend: {
                    position: 'top'
                },
                tooltip: {
                    callbacks: {
                        label: function(context) {
                            const data = context.raw;
                            return [
                                context.dataset.label,
                                `收益率: ${data.y.toFixed(1)}%`,
                                `持仓天数: ${data.x.toFixed(1)}天`,
                                `成功率: ${(data.r / 20 * 100).toFixed(1)}%`
                            ];
                        }
                    }
                }
            }
        }
    });
}
// 格式化策略详情
function formatStrategyDetails(strategy) {
    const isLimitPrice = strategy.signal_name.includes('限价');
    
    // 创建详情容器
    let detailsHtml = `
        <div class="strategy-details p-3">
            <div class="row">
                <div class="col-md-6">
                    <h6 class="mb-3">策略详情</h6>
                    <ul class="list-unstyled">
                        <li><strong>选股策略:</strong> ${strategy.strategy_name}</li>
                        <li><strong>买入信号:</strong> ${strategy.signal_name}</li>
                        <li><strong>目标设置:</strong> ${strategy.target_name}</li>
                    </ul>
                </div>
                <div class="col-md-6">
                    <h6 class="mb-3">性能指标</h6>
                    <ul class="list-unstyled">
                        <li><strong>成功率:</strong> ${(strategy.performance.success_rate * 100).toFixed(1)}%</li>
                        <li><strong>止损率:</strong> ${(strategy.performance.stop_loss_rate * 100).toFixed(1)}%</li>
                        <li><strong>止损失败率:</strong> ${(strategy.performance.stop_loss_fail_rate * 100).toFixed(1)}%</li>
                        <li><strong>平均收益:</strong> ${(strategy.performance.avg_return * 100).toFixed(1)}%</li>
                        <li><strong>平均持仓:</strong> ${strategy.performance.avg_hold_days.toFixed(1)}天</li>
                    </ul>
                </div>
            </div>
            
            <h6 class="mt-3 mb-2">推荐股票</h6>
    `;
    
    // 添加股票表格
    if (strategy.recommendations && strategy.recommendations.length > 0) {
        detailsHtml += `
            <div class="table-responsive">
                <table class="table table-sm table-bordered">
                    <thead class="table-light">
                        <tr>
                            <th>代码</th>
                            <th>名称</th>
                            ${isLimitPrice ? '<th>前收盘</th>' : ''}
                            <th>买入价</th>
                            <th>目标价</th>
                            <th>止损价</th>
                        </tr>
                    </thead>
                    <tbody>
        `;
        
        // 添加股票数据
        strategy.recommendations.forEach(stock => {
            detailsHtml += `
                <tr>
                    <td>${stock.symbol}</td>
                    <td>${stock.name}</td>
                    ${isLimitPrice ? `<td>${stock.prev_close?.toFixed(2) || '-'}</td>` : ''}
                    <td>${stock.buy_price.toFixed(2)}</td>
                    <td>${stock.target_price.toFixed(2)}</td>
                    <td>${stock.stop_loss_price.toFixed(2)}</td>
                </tr>
            `;
        });
        
        detailsHtml += `
                    </tbody>
                </table>
            </div>
        `;
    } else {
        detailsHtml += `<p class="text-muted">暂无推荐股票</p>`;
    }
    
    detailsHtml += `</div>`;
    
    return detailsHtml;
}
