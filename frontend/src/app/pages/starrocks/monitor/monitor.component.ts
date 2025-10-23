import { Component, OnInit, OnDestroy } from '@angular/core';

import { NbToastrService } from '@nebular/theme';
import { Subject } from 'rxjs';
import { takeUntil } from 'rxjs/operators';
import { ClusterContextService } from '../../../@core/data/cluster-context.service';
import { Cluster } from '../../../@core/data/cluster.service';

@Component({
  selector: 'ngx-monitor',
  templateUrl: './monitor.component.html',
  styleUrls: ['./monitor.component.scss'],
})
export class MonitorComponent implements OnInit, OnDestroy {
  clusterId: number;
  activeCluster: Cluster | null = null;
  loading = false;
  metrics: any = null;
  private destroy$ = new Subject<void>();

  // Chart options
  qpsRpsChartOption: any = {};
  latencyChartOption: any = {};
  cpuChartOption: any = {};
  memoryChartOption: any = {};
  diskChartOption: any = {};

  constructor(
    
    private toastrService: NbToastrService,
    private clusterContext: ClusterContextService,
  ) {
    // Get clusterId from ClusterContextService
    this.clusterId = this.clusterContext.getActiveClusterId() || 0;
  }

  ngOnInit(): void {
    // Subscribe to active cluster changes
    this.clusterContext.activeCluster$
      .pipe(takeUntil(this.destroy$))
      .subscribe(cluster => {
        this.activeCluster = cluster;
        if (cluster) {
          // Always use the active cluster (override route parameter)
          const newClusterId = cluster.id;
          if (this.clusterId !== newClusterId) {
            console.log('[Monitor] Switching cluster from', this.clusterId, 'to', newClusterId);
            this.clusterId = newClusterId;
            this.loadMetrics();
          }
        }
      });

    // Load metrics if clusterId is already set
    if (this.clusterId && this.clusterId > 0) {
      this.loadMetrics();
    } else if (!this.clusterContext.hasActiveCluster()) {
      this.toastrService.warning('请先在集群概览页面激活一个集群', '提示');
    }
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  loadMetrics(): void {
    this.loading = true;
    
    // Mock data - replace with actual API call
    setTimeout(() => {
      this.metrics = {
        qps: 0.00,
        rps: 0.00,
        query_total: 0,
        query_success: 0,
        query_err: 0,
        query_latency_p50: 0.00,
        query_latency_p95: 0.00,
        query_latency_p99: 0.00,
        success_rate: 99.92,
        backend_total: 11,
        backend_alive: 11,
        jvm_heap_total: 8589934592,
        jvm_heap_used: 2147483648,
        jvm_heap_usage_pct: 25.0,
        disk_total_bytes: 1099511627776,
        disk_used_bytes: 109951162777,
        disk_usage_pct: 10.0,
        cpu_usage_pct: 0.0,
        memory_usage_pct: 0.0,
        running_queries: 0,
      };
      
      this.initCharts();
      this.loading = false;
    }, 500);
  }

  initCharts(): void {
    const textColor = '#8f9bb3';
    const gridColor = '#edf1f7';

    // QPS & RPS Chart
    this.qpsRpsChartOption = {
      backgroundColor: 'transparent',
      color: ['#3366ff', '#00d68f'],
      tooltip: {
        trigger: 'axis',
        axisPointer: {
          type: 'cross',
        },
      },
      legend: {
        data: ['QPS', 'RPS'],
        textStyle: {
          color: textColor,
        },
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '3%',
        containLabel: true,
      },
      xAxis: {
        type: 'category',
        boundaryGap: false,
        data: ['当前'],
        axisLine: {
          lineStyle: {
            color: gridColor,
          },
        },
        axisLabel: {
          color: textColor,
        },
      },
      yAxis: {
        type: 'value',
        axisLine: {
          lineStyle: {
            color: gridColor,
          },
        },
        axisLabel: {
          color: textColor,
        },
        splitLine: {
          lineStyle: {
            color: gridColor,
          },
        },
      },
      series: [
        {
          name: 'QPS',
          type: 'line',
          smooth: true,
          data: [this.metrics.qps],
          areaStyle: {
            opacity: 0.3,
          },
        },
        {
          name: 'RPS',
          type: 'line',
          smooth: true,
          data: [this.metrics.rps],
          areaStyle: {
            opacity: 0.3,
          },
        },
      ],
    };

    // Query Latency Chart (P50/P95/P99)
    this.latencyChartOption = {
      backgroundColor: 'transparent',
      color: ['#00d68f', '#ffaa00', '#ff3d71'],
      tooltip: {
        trigger: 'axis',
        axisPointer: {
          type: 'shadow',
        },
      },
      legend: {
        data: ['P50', 'P95', 'P99'],
        textStyle: {
          color: textColor,
        },
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '3%',
        containLabel: true,
      },
      xAxis: {
        type: 'category',
        data: ['延迟'],
        axisLine: {
          lineStyle: {
            color: gridColor,
          },
        },
        axisLabel: {
          color: textColor,
        },
      },
      yAxis: {
        type: 'value',
        name: 'ms',
        axisLine: {
          lineStyle: {
            color: gridColor,
          },
        },
        axisLabel: {
          color: textColor,
        },
        splitLine: {
          lineStyle: {
            color: gridColor,
          },
        },
      },
      series: [
        {
          name: 'P50',
          type: 'bar',
          data: [this.metrics.query_latency_p50],
        },
        {
          name: 'P95',
          type: 'bar',
          data: [this.metrics.query_latency_p95],
        },
        {
          name: 'P99',
          type: 'bar',
          data: [this.metrics.query_latency_p99],
        },
      ],
    };

    // CPU/Memory/Disk Usage Chart
    this.cpuChartOption = {
      backgroundColor: 'transparent',
      color: ['#3366ff', '#00d68f', '#ffaa00'],
      tooltip: {
        trigger: 'item',
      },
      legend: {
        orient: 'vertical',
        left: 'left',
        textStyle: {
          color: textColor,
        },
        data: ['CPU', '内存', '磁盘'],
      },
      series: [
        {
          name: '资源使用率',
          type: 'pie',
          radius: ['50%', '70%'],
          avoidLabelOverlap: false,
          label: {
            show: true,
            formatter: '{b}: {d}%',
            color: textColor,
          },
          emphasis: {
            label: {
              show: true,
              fontSize: '14',
              fontWeight: 'bold',
            },
          },
          labelLine: {
            show: true,
          },
          data: [
            { value: this.metrics.cpu_usage_pct || 0, name: 'CPU' },
            { value: this.metrics.memory_usage_pct || 0, name: '内存' },
            { value: this.metrics.disk_usage_pct, name: '磁盘' },
          ],
        },
      ],
    };
  }

  formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  getProgressStatus(percentage: number): string {
    if (percentage > 80) return 'danger';
    if (percentage > 60) return 'warning';
    return 'success';
  }
}
