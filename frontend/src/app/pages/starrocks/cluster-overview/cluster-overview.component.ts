import { Component, OnInit, OnDestroy } from '@angular/core';
import { Router } from '@angular/router';
import { Subject, interval } from 'rxjs';
import { takeUntil, switchMap } from 'rxjs/operators';
import {
  OverviewService,
  ClusterOverview,
  HealthCard,
  PerformanceTrends,
  ResourceTrends,
  DataStatistics,
  CapacityPrediction,
  TopTableBySize,
  TopTableByAccess,
  SlowQuery,
} from '../../../@core/data/overview.service';
import { ClusterContextService } from '../../../@core/data/cluster-context.service';

@Component({
  selector: 'ngx-cluster-overview',
  templateUrl: './cluster-overview.component.html',
  styleUrls: ['./cluster-overview.component.scss'],
})
export class ClusterOverviewComponent implements OnInit, OnDestroy {
  overview: ClusterOverview | null = null;
  healthCards: HealthCard[] = [];
  performanceTrends: PerformanceTrends | null = null;
  resourceTrends: ResourceTrends | null = null;
  dataStatistics: DataStatistics | null = null;
  capacityPrediction: CapacityPrediction | null = null;
  
  clusterId: number = 0;
  timeRange: string = '1h';
  loading = false;
  autoRefresh = true;
  refreshInterval = 30; // seconds
  
  private destroy$ = new Subject<void>();

  // Time range options
  timeRangeOptions = [
    { label: '1 Hour', value: '1h' },
    { label: '6 Hours', value: '6h' },
    { label: '24 Hours', value: '24h' },
    { label: '3 Days', value: '3d' },
  ];

  // Refresh interval options
  refreshIntervalOptions = [
    { label: '15s', value: 15 },
    { label: '30s', value: 30 },
    { label: '1m', value: 60 },
    { label: 'Manual', value: 0 },
  ];

  constructor(
    private overviewService: OverviewService,
    private clusterContext: ClusterContextService,
    private router: Router,
  ) {}

  ngOnInit() {
    // Listen to active cluster changes
    this.clusterContext.activeCluster$
      .pipe(takeUntil(this.destroy$))
      .subscribe(cluster => {
        if (cluster) {
          this.clusterId = cluster.id;
          this.loadOverview();
          this.setupAutoRefresh();
        }
      });
  }

  ngOnDestroy() {
    this.destroy$.next();
    this.destroy$.complete();
  }

  setupAutoRefresh() {
    if (this.refreshInterval > 0) {
      interval(this.refreshInterval * 1000)
        .pipe(
          takeUntil(this.destroy$),
        )
        .subscribe(() => {
          if (this.autoRefresh) {
            this.loadOverview(false); // silent refresh
          }
        });
    }
  }

  loadOverview(showLoading: boolean = true) {
    if (showLoading) {
      this.loading = true;
    }

    // Load all data in parallel
    Promise.all([
      this.overviewService.getHealthCards(this.clusterId).toPromise(),
      this.overviewService.getPerformanceTrends(this.clusterId, this.timeRange).toPromise(),
      this.overviewService.getResourceTrends(this.clusterId, this.timeRange).toPromise(),
      this.overviewService.getDataStatistics(this.clusterId).toPromise(),
      this.overviewService.getCapacityPrediction(this.clusterId).toPromise(),
    ])
      .then(([healthCards, performanceTrends, resourceTrends, dataStatistics, capacityPrediction]) => {
        this.healthCards = healthCards || [];
        this.performanceTrends = performanceTrends;
        this.resourceTrends = resourceTrends;
        this.dataStatistics = dataStatistics;
        this.capacityPrediction = capacityPrediction;
        this.loading = false;
      })
      .catch(err => {
        console.error('Failed to load cluster overview:', err);
        this.loading = false;
      });
  }

  onTimeRangeChange(range: string) {
    this.timeRange = range;
    this.loadOverview();
  }

  onRefreshIntervalChange(interval: number) {
    this.refreshInterval = interval;
    this.autoRefresh = interval > 0;
    // Restart auto-refresh
    this.destroy$.next();
    this.setupAutoRefresh();
  }

  onManualRefresh() {
    this.loadOverview();
  }

  onToggleAutoRefresh() {
    this.autoRefresh = !this.autoRefresh;
  }

  // Navigation methods
  navigateToCard(card: HealthCard) {
    if (card.navigateTo) {
      this.router.navigate([card.navigateTo]);
    }
  }

  navigateToQueries() {
    this.router.navigate(['/pages/starrocks/queries']);
  }

  navigateToBackends() {
    this.router.navigate(['/pages/starrocks/backends']);
  }

  navigateToMaterializedViews() {
    this.router.navigate(['/pages/starrocks/materialized-views']);
  }

  // Helper methods
  formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  formatNumber(num: number): string {
    if (num >= 1000000) {
      return (num / 1000000).toFixed(2) + 'M';
    } else if (num >= 1000) {
      return (num / 1000).toFixed(2) + 'K';
    }
    return num.toString();
  }

  formatDuration(ms: number): string {
    if (ms < 1000) return ms + 'ms';
    if (ms < 60000) return (ms / 1000).toFixed(2) + 's';
    if (ms < 3600000) return (ms / 60000).toFixed(2) + 'm';
    return (ms / 3600000).toFixed(2) + 'h';
  }

  getStatusIcon(status: string): string {
    switch (status) {
      case 'success': return 'checkmark-circle-2-outline';
      case 'warning': return 'alert-triangle-outline';
      case 'danger': return 'close-circle-outline';
      case 'info': return 'info-outline';
      default: return 'info-outline';
    }
  }

  getTrendIcon(trend: number): string {
    if (trend > 0) return 'trending-up-outline';
    if (trend < 0) return 'trending-down-outline';
    return 'minus-outline';
  }

  getTrendColor(trend: number): string {
    if (trend > 0) return 'success';
    if (trend < 0) return 'danger';
    return 'basic';
  }

  // ECharts Configuration Methods (使用 ngx-admin 兼容的渐变样式)

  private getBaseChartOptions(color: string): any {
    return {
      grid: {
        left: '3%',
        right: '4%',
        bottom: '3%',
        top: '10%',
        containLabel: true,
      },
      tooltip: {
        trigger: 'axis',
        axisPointer: {
          type: 'cross',
        },
      },
      xAxis: {
        type: 'category',
        boundaryGap: false,
      },
      yAxis: {
        type: 'value',
      },
    };
  }

  getQpsChartOptions(): any {
    if (!this.performanceTrends || !this.performanceTrends.qpsSeries) {
      return {};
    }

    const data = this.performanceTrends.qpsSeries;
    const times = data.map(d => new Date(d.timestamp).toLocaleTimeString());
    const values = data.map(d => d.value);

    return {
      ...this.getBaseChartOptions('#36f'),
      xAxis: {
        ...this.getBaseChartOptions('#36f').xAxis,
        data: times,
      },
      series: [
        {
          name: 'QPS',
          type: 'line',
          smooth: true,
          symbol: 'circle',
          symbolSize: 6,
          sampling: 'lttb',
          itemStyle: {
            color: '#36f',
          },
          areaStyle: {
            color: {
              type: 'linear',
              x: 0,
              y: 0,
              x2: 0,
              y2: 1,
              colorStops: [
                { offset: 0, color: 'rgba(51, 102, 255, 0.3)' },
                { offset: 1, color: 'rgba(51, 102, 255, 0.05)' },
              ],
            },
          },
          data: values,
        },
      ],
    };
  }

  getLatencyChartOptions(): any {
    if (!this.performanceTrends || !this.performanceTrends.latencyP99Series) {
      return {};
    }

    const data = this.performanceTrends.latencyP99Series;
    const times = data.map(d => new Date(d.timestamp).toLocaleTimeString());
    const values = data.map(d => d.value);

    return {
      ...this.getBaseChartOptions('#f90'),
      xAxis: {
        ...this.getBaseChartOptions('#f90').xAxis,
        data: times,
      },
      series: [
        {
          name: 'P99 Latency (ms)',
          type: 'line',
          smooth: true,
          symbol: 'circle',
          symbolSize: 6,
          sampling: 'lttb',
          itemStyle: {
            color: '#f90',
          },
          areaStyle: {
            color: {
              type: 'linear',
              x: 0,
              y: 0,
              x2: 0,
              y2: 1,
              colorStops: [
                { offset: 0, color: 'rgba(255, 153, 0, 0.3)' },
                { offset: 1, color: 'rgba(255, 153, 0, 0.05)' },
              ],
            },
          },
          data: values,
        },
      ],
    };
  }

  getErrorRateChartOptions(): any {
    if (!this.performanceTrends || !this.performanceTrends.errorRateSeries) {
      return {};
    }

    const data = this.performanceTrends.errorRateSeries;
    const times = data.map(d => new Date(d.timestamp).toLocaleTimeString());
    const values = data.map(d => d.value);

    return {
      ...this.getBaseChartOptions('#f33'),
      xAxis: {
        ...this.getBaseChartOptions('#f33').xAxis,
        data: times,
      },
      series: [
        {
          name: 'Error Rate (%)',
          type: 'line',
          smooth: true,
          symbol: 'circle',
          symbolSize: 6,
          sampling: 'lttb',
          itemStyle: {
            color: '#f33',
          },
          areaStyle: {
            color: {
              type: 'linear',
              x: 0,
              y: 0,
              x2: 0,
              y2: 1,
              colorStops: [
                { offset: 0, color: 'rgba(255, 51, 51, 0.3)' },
                { offset: 1, color: 'rgba(255, 51, 51, 0.05)' },
              ],
            },
          },
          data: values,
        },
      ],
    };
  }

  getCpuChartOptions(): any {
    if (!this.resourceTrends || !this.resourceTrends.cpuUsageSeries) {
      return {};
    }

    const data = this.resourceTrends.cpuUsageSeries;
    const times = data.map(d => new Date(d.timestamp).toLocaleTimeString());
    const values = data.map(d => d.value);

    return {
      ...this.getBaseChartOptions('#3c6'),
      xAxis: {
        ...this.getBaseChartOptions('#3c6').xAxis,
        data: times,
      },
      yAxis: {
        type: 'value',
        max: 100,
      },
      series: [
        {
          name: 'CPU Usage (%)',
          type: 'line',
          smooth: true,
          symbol: 'circle',
          symbolSize: 6,
          sampling: 'lttb',
          itemStyle: {
            color: '#3c6',
          },
          areaStyle: {
            color: {
              type: 'linear',
              x: 0,
              y: 0,
              x2: 0,
              y2: 1,
              colorStops: [
                { offset: 0, color: 'rgba(51, 204, 102, 0.3)' },
                { offset: 1, color: 'rgba(51, 204, 102, 0.05)' },
              ],
            },
          },
          data: values,
        },
      ],
    };
  }

  getMemoryChartOptions(): any {
    if (!this.resourceTrends || !this.resourceTrends.memoryUsageSeries) {
      return {};
    }

    const data = this.resourceTrends.memoryUsageSeries;
    const times = data.map(d => new Date(d.timestamp).toLocaleTimeString());
    const values = data.map(d => d.value);

    return {
      ...this.getBaseChartOptions('#c6f'),
      xAxis: {
        ...this.getBaseChartOptions('#c6f').xAxis,
        data: times,
      },
      yAxis: {
        type: 'value',
        max: 100,
      },
      series: [
        {
          name: 'Memory Usage (%)',
          type: 'line',
          smooth: true,
          symbol: 'circle',
          symbolSize: 6,
          sampling: 'lttb',
          itemStyle: {
            color: '#c6f',
          },
          areaStyle: {
            color: {
              type: 'linear',
              x: 0,
              y: 0,
              x2: 0,
              y2: 1,
              colorStops: [
                { offset: 0, color: 'rgba(204, 102, 255, 0.3)' },
                { offset: 1, color: 'rgba(204, 102, 255, 0.05)' },
              ],
            },
          },
          data: values,
        },
      ],
    };
  }

  getDiskChartOptions(): any {
    if (!this.resourceTrends || !this.resourceTrends.diskUsageSeries) {
      return {};
    }

    const data = this.resourceTrends.diskUsageSeries;
    const times = data.map(d => new Date(d.timestamp).toLocaleTimeString());
    const values = data.map(d => d.value);

    return {
      ...this.getBaseChartOptions('#fc6'),
      xAxis: {
        ...this.getBaseChartOptions('#fc6').xAxis,
        data: times,
      },
      yAxis: {
        type: 'value',
        max: 100,
      },
      series: [
        {
          name: 'Disk Usage (%)',
          type: 'line',
          smooth: true,
          symbol: 'circle',
          symbolSize: 6,
          sampling: 'lttb',
          itemStyle: {
            color: '#fc6',
          },
          areaStyle: {
            color: {
              type: 'linear',
              x: 0,
              y: 0,
              x2: 0,
              y2: 1,
              colorStops: [
                { offset: 0, color: 'rgba(255, 204, 102, 0.3)' },
                { offset: 1, color: 'rgba(255, 204, 102, 0.05)' },
              ],
            },
          },
          data: values,
        },
      ],
    };
  }

  getNetworkChartOptions(): any {
    if (!this.resourceTrends || !this.resourceTrends.networkTxSeries || !this.resourceTrends.networkRxSeries) {
      return {};
    }

    const txData = this.resourceTrends.networkTxSeries;
    const rxData = this.resourceTrends.networkRxSeries;
    const times = txData.map(d => new Date(d.timestamp).toLocaleTimeString());
    const txValues = txData.map(d => d.value / 1024 / 1024); // Convert to MB/s
    const rxValues = rxData.map(d => d.value / 1024 / 1024);

    return {
      ...this.getBaseChartOptions('#36f'),
      xAxis: {
        ...this.getBaseChartOptions('#36f').xAxis,
        data: times,
      },
      legend: {
        data: ['TX (Send)', 'RX (Receive)'],
      },
      series: [
        {
          name: 'TX (Send)',
          type: 'line',
          smooth: true,
          symbol: 'circle',
          symbolSize: 6,
          sampling: 'lttb',
          itemStyle: {
            color: '#36f',
          },
          areaStyle: {
            color: {
              type: 'linear',
              x: 0,
              y: 0,
              x2: 0,
              y2: 1,
              colorStops: [
                { offset: 0, color: 'rgba(51, 102, 255, 0.3)' },
                { offset: 1, color: 'rgba(51, 102, 255, 0.05)' },
              ],
            },
          },
          data: txValues,
        },
        {
          name: 'RX (Receive)',
          type: 'line',
          smooth: true,
          symbol: 'circle',
          symbolSize: 6,
          sampling: 'lttb',
          itemStyle: {
            color: '#3c6',
          },
          areaStyle: {
            color: {
              type: 'linear',
              x: 0,
              y: 0,
              x2: 0,
              y2: 1,
              colorStops: [
                { offset: 0, color: 'rgba(51, 204, 102, 0.3)' },
                { offset: 1, color: 'rgba(51, 204, 102, 0.05)' },
              ],
            },
          },
          data: rxValues,
        },
      ],
    };
  }

  getIoChartOptions(): any {
    if (!this.resourceTrends || !this.resourceTrends.ioReadSeries || !this.resourceTrends.ioWriteSeries) {
      return {};
    }

    const readData = this.resourceTrends.ioReadSeries;
    const writeData = this.resourceTrends.ioWriteSeries;
    const times = readData.map(d => new Date(d.timestamp).toLocaleTimeString());
    const readValues = readData.map(d => d.value / 1024 / 1024); // Convert to MB/s
    const writeValues = writeData.map(d => d.value / 1024 / 1024);

    return {
      ...this.getBaseChartOptions('#f90'),
      xAxis: {
        ...this.getBaseChartOptions('#f90').xAxis,
        data: times,
      },
      legend: {
        data: ['Read', 'Write'],
      },
      series: [
        {
          name: 'Read',
          type: 'line',
          smooth: true,
          symbol: 'circle',
          symbolSize: 6,
          sampling: 'lttb',
          itemStyle: {
            color: '#3c6',
          },
          areaStyle: {
            color: {
              type: 'linear',
              x: 0,
              y: 0,
              x2: 0,
              y2: 1,
              colorStops: [
                { offset: 0, color: 'rgba(51, 204, 102, 0.3)' },
                { offset: 1, color: 'rgba(51, 204, 102, 0.05)' },
              ],
            },
          },
          data: readValues,
        },
        {
          name: 'Write',
          type: 'line',
          smooth: true,
          symbol: 'circle',
          symbolSize: 6,
          sampling: 'lttb',
          itemStyle: {
            color: '#f90',
          },
          areaStyle: {
            color: {
              type: 'linear',
              x: 0,
              y: 0,
              x2: 0,
              y2: 1,
              colorStops: [
                { offset: 0, color: 'rgba(255, 153, 0, 0.3)' },
                { offset: 1, color: 'rgba(255, 153, 0, 0.05)' },
              ],
            },
          },
          data: writeValues,
        },
      ],
    };
  }
}

