import { Component, OnInit, OnDestroy } from '@angular/core';
import { interval, Subject } from 'rxjs';
import { takeUntil, switchMap } from 'rxjs/operators';
import { NbToastrService } from '@nebular/theme';
import { LocalDataSource } from 'ng2-smart-table';
import { NodeService, Backend } from '../../../@core/data/node.service';
import { ClusterService, Cluster } from '../../../@core/data/cluster.service';
import { ClusterContextService } from '../../../@core/data/cluster-context.service';
import { ErrorHandler } from '../../../@core/utils/error-handler';

@Component({
  selector: 'ngx-backends',
  templateUrl: './backends.component.html',
  styleUrls: ['./backends.component.scss'],
})
export class BackendsComponent implements OnInit, OnDestroy {
  source: LocalDataSource = new LocalDataSource();
  clusterId: number;
  activeCluster: Cluster | null = null;
  clusterName: string = '';
  loading = true;
  autoRefresh = true;
  private destroy$ = new Subject<void>();

  settings = {
    mode: 'external',
    hideSubHeader: false, // Enable search
    noDataMessage: '暂无Backend节点数据',
    actions: false,
    pager: {
      display: true,
      perPage: 15,
    },
    columns: {
      BackendId: {
        title: 'BE ID',
        type: 'string',
        width: '8%',
      },
      IP: {
        title: '主机',
        type: 'string',
      },
      HeartbeatPort: {
        title: '心跳端口',
        type: 'string',
        width: '10%',
      },
      BePort: {
        title: 'BE 端口',
        type: 'string',
        width: '10%',
      },
      Alive: {
        title: '状态',
        type: 'html',
        width: '8%',
        valuePrepareFunction: (value: string) => {
          const status = value === 'true' ? 'success' : 'danger';
          const text = value === 'true' ? '在线' : '离线';
          return `<span class="badge badge-${status}">${text}</span>`;
        },
      },
      TabletNum: {
        title: 'Tablet 数',
        type: 'string',
        width: '10%',
      },
      DataUsedCapacity: {
        title: '已用存储',
        type: 'string',
      },
      TotalCapacity: {
        title: '总存储',
        type: 'string',
      },
      UsedPct: {
        title: '磁盘使用率',
        type: 'string',
        width: '10%',
      },
      CpuUsedPct: {
        title: 'CPU 使用率',
        type: 'string',
        width: '12%',
      },
      MemUsedPct: {
        title: '内存使用率',
        type: 'string',
        width: '10%',
      },
      NumRunningQueries: {
        title: '运行查询数',
        type: 'string',
        width: '10%',
      },
    },
  };

  constructor(
    private nodeService: NodeService,
    private clusterService: ClusterService,
    private clusterContext: ClusterContextService,
    private toastrService: NbToastrService,
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
          const newClusterId = cluster.id;
          if (this.clusterId !== newClusterId) {
            console.log('[Backends] Active cluster changed, switching from', this.clusterId, 'to', newClusterId);
            this.clusterId = newClusterId;
            this.loadClusterInfo();
            this.loadBackends();
          }
        }
      });

    // Load data if clusterId is already set
    if (this.clusterId && this.clusterId > 0) {
      this.loadClusterInfo();
      this.loadBackends();
    } else {
      this.toastrService.warning('请先在集群概览页面激活一个集群', '提示');
      this.loading = false;
    }
    
    // Auto refresh every 10 seconds
    interval(10000)
      .pipe(
        takeUntil(this.destroy$),
        switchMap(() => this.nodeService.listBackends(this.clusterId)),
      )
      .subscribe({
        next: (backends) => {
          if (this.autoRefresh) {
            this.source.load(backends);
          }
        },
        error: (error) => console.error('Auto refresh error:', error),
      });
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  loadClusterInfo(): void {
    this.clusterService.getCluster(this.clusterId).subscribe({
      next: (cluster) => {
        this.clusterName = cluster.name;
      },
      error: (error) => {
        console.error('Load cluster error:', error);
      },
    });
  }

  loadBackends(): void {
    this.loading = true;
    this.nodeService.listBackends(this.clusterId).subscribe({
      next: (backends) => {
        this.source.load(backends);
        this.loading = false;
      },
      error: (error) => {
        this.toastrService.danger(
          ErrorHandler.extractErrorMessage(error),
          '错误',
        );
        this.loading = false;
      },
    });
  }

  toggleAutoRefresh(): void {
    this.autoRefresh = !this.autoRefresh;
    if (this.autoRefresh) {
      this.toastrService.info('已启用自动刷新', '提示');
      this.loadBackends();
    } else {
      this.toastrService.info('已禁用自动刷新', '提示');
    }
  }
}