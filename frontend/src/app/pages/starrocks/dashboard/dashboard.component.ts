import { Component, OnInit, OnDestroy } from '@angular/core';
import { Router } from '@angular/router';
import { interval, Subject } from 'rxjs';
import { takeUntil, switchMap } from 'rxjs/operators';
import { NbToastrService } from '@nebular/theme';
import { ClusterService, Cluster, ClusterHealth } from '../../../@core/data/cluster.service';
import { ClusterContextService } from '../../../@core/data/cluster-context.service';
import { ErrorHandler } from '../../../@core/utils/error-handler';

interface ClusterCard {
  cluster: Cluster;
  health?: ClusterHealth;
  loading: boolean;
  isActive: boolean;
}

@Component({
  selector: 'ngx-dashboard',
  templateUrl: './dashboard.component.html',
  styleUrls: ['./dashboard.component.scss'],
})
export class DashboardComponent implements OnInit, OnDestroy {
  clusters: ClusterCard[] = [];
  loading = true;
  activeCluster: Cluster | null = null;
  private destroy$ = new Subject<void>();

  constructor(
    private clusterService: ClusterService,
    private clusterContext: ClusterContextService,
    private toastrService: NbToastrService,
    private router: Router,
  ) {}

  ngOnInit(): void {
    // Subscribe to active cluster changes
    this.clusterContext.activeCluster$
      .pipe(takeUntil(this.destroy$))
      .subscribe(cluster => {
        this.activeCluster = cluster;
        this.updateActiveStatus();
      });

    this.loadClusters();
    
    // Auto refresh every 30 seconds
    interval(30000)
      .pipe(
        takeUntil(this.destroy$),
        switchMap(() => this.clusterService.listClusters()),
      )
      .subscribe({
        next: (clusters) => {
          this.updateClusters(clusters);
          this.loadHealthStatus(); // 重新加载健康状态
        },
        error: (error) => this.handleError(error),
      });
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  loadClusters(): void {
    this.loading = true;
    this.clusterService.listClusters().subscribe({
      next: (clusters) => {
        this.updateClusters(clusters);
        this.loadHealthStatus();
        
        // Try to restore active cluster from localStorage
        const savedClusterId = this.clusterContext.getSavedClusterId();
        console.log('[Dashboard] Saved cluster ID from localStorage:', savedClusterId);
        
        if (savedClusterId && !this.activeCluster) {
          const savedCluster = clusters.find(c => c.id === savedClusterId);
          if (savedCluster) {
            console.log('[Dashboard] Restoring saved cluster:', savedCluster);
            this.clusterContext.setActiveCluster(savedCluster);
            this.toastrService.success(`已恢复激活集群: ${savedCluster.name}`, '提示');
          } else {
            console.log('[Dashboard] Saved cluster not found, clearing saved ID');
            // Saved cluster no longer exists, clear localStorage
            this.clusterContext.clearActiveCluster();
          }
        } else if (clusters.length === 1 && !this.activeCluster) {
          // Auto-activate the first cluster if:
          // 1. There is exactly one cluster
          // 2. No cluster is currently active
          console.log('[Dashboard] Auto-activating single cluster:', clusters[0]);
          this.clusterContext.setActiveCluster(clusters[0]);
          this.toastrService.success(`已自动激活集群: ${clusters[0].name}`, '提示');
        }
        
        this.loading = false;
      },
      error: (error) => {
        this.handleError(error);
        this.loading = false;
      },
    });
  }

  updateClusters(clusters: Cluster[]): void {
    const activeId = this.activeCluster?.id;
    this.clusters = clusters.map((cluster) => ({
      cluster,
      loading: false,
      isActive: cluster.id === activeId,
    }));
  }

  updateActiveStatus(): void {
    const activeId = this.activeCluster?.id;
    this.clusters.forEach(card => {
      card.isActive = card.cluster.id === activeId;
    });
  }

  toggleActiveCluster(clusterCard: ClusterCard): void {
    const isCurrentlyActive = this.activeCluster?.id === clusterCard.cluster.id;
    
    if (isCurrentlyActive) {
      // Deactivate current cluster
      console.log('[Dashboard] Deactivating cluster:', clusterCard.cluster);
      this.clusterContext.clearActiveCluster();
      this.toastrService.info('已取消激活集群', '提示');
    } else {
      // Activate new cluster (automatically deactivates previous)
      console.log('[Dashboard] Activating cluster:', clusterCard.cluster);
      this.clusterContext.setActiveCluster(clusterCard.cluster);
      this.toastrService.success(`已激活集群: ${clusterCard.cluster.name}`, '成功');
    }
  }

  loadHealthStatus(): void {
    this.clusters.forEach((clusterCard) => {
      clusterCard.loading = true;
      this.clusterService.getHealth(clusterCard.cluster.id).subscribe({
        next: (health) => {
          clusterCard.health = health;
          clusterCard.loading = false;
        },
        error: () => {
          clusterCard.loading = false;
        },
      });
    });
  }

  getStatusColor(status?: string): string {
    switch (status) {
      case 'healthy':
        return 'success';  // 绿色 - 健康
      case 'warning':
        return 'warning';  // 黄色 - 警告
      case 'critical':
        return 'danger';   // 红色 - 危险/不健康
      default:
        return 'basic';    // 默认 - 未知状态
    }
  }

  navigateToCluster(clusterId: number): void {
    // 先设置激活集群，然后导航到集群列表
    const cluster = this.clusters.find(c => c.cluster.id === clusterId)?.cluster;
    if (cluster) {
      this.clusterContext.setActiveCluster(cluster);
    }
    this.router.navigate(['/pages/starrocks/clusters']);
  }

  navigateToBackends(clusterId: number): void {
    // 先设置激活集群，然后导航到后端节点页面
    const cluster = this.clusters.find(c => c.cluster.id === clusterId)?.cluster;
    if (cluster) {
      this.clusterContext.setActiveCluster(cluster);
    }
    this.router.navigate(['/pages/starrocks/backends']);
  }

  navigateToFrontends(clusterId: number): void {
    // 先设置激活集群，然后导航到前端节点页面
    const cluster = this.clusters.find(c => c.cluster.id === clusterId)?.cluster;
    if (cluster) {
      this.clusterContext.setActiveCluster(cluster);
    }
    this.router.navigate(['/pages/starrocks/frontends']);
  }

  navigateToMonitor(clusterId: number): void {
    // 先设置激活集群，然后导航到监控页面
    const cluster = this.clusters.find(c => c.cluster.id === clusterId)?.cluster;
    if (cluster) {
      this.clusterContext.setActiveCluster(cluster);
    }
    this.router.navigate(['/pages/starrocks/monitor']);
  }

  navigateToQueries(clusterId: number): void {
    // 先设置激活集群，然后导航到查询页面
    const cluster = this.clusters.find(c => c.cluster.id === clusterId)?.cluster;
    if (cluster) {
      this.clusterContext.setActiveCluster(cluster);
    }
    this.router.navigate(['/pages/starrocks/queries']);
  }

  addCluster(): void {
    this.router.navigate(['/pages/starrocks/clusters/new']);
  }

  private handleError(error: any): void {
    console.error('Error:', error);
    this.toastrService.danger(
      ErrorHandler.extractErrorMessage(error),
      '错误',
    );
  }
}

