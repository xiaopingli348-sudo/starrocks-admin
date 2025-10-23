import { Component, OnInit, OnDestroy } from '@angular/core';
import { Router } from '@angular/router';
import { Subject } from 'rxjs';
import { takeUntil } from 'rxjs/operators';
import { NbToastrService } from '@nebular/theme';
import { ClusterService, Cluster } from '../../../@core/data/cluster.service';
import { ClusterContextService } from '../../../@core/data/cluster-context.service';

@Component({
  selector: 'ngx-cluster-selector',
  templateUrl: './cluster-selector.component.html',
  styleUrls: ['./cluster-selector.component.scss'],
})
export class ClusterSelectorComponent implements OnInit, OnDestroy {
  clusters: Cluster[] = [];
  activeCluster: Cluster | null = null;
  loading = false;
  private destroy$ = new Subject<void>();

  constructor(
    private clusterService: ClusterService,
    private clusterContext: ClusterContextService,
    private router: Router,
    private toastr: NbToastrService,
  ) {}

  ngOnInit(): void {
    // Subscribe to active cluster changes
    this.clusterContext.activeCluster$
      .pipe(takeUntil(this.destroy$))
      .subscribe(cluster => {
        this.activeCluster = cluster;
      });

    // Load clusters
    this.loadClusters();
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  loadClusters(): void {
    this.loading = true;
    this.clusterService.listClusters().subscribe({
      next: (clusters) => {
        this.clusters = clusters;
        this.loading = false;

        // Auto-select cluster if none is active
        if (clusters.length > 0 && !this.activeCluster) {
          const savedId = this.clusterContext.getSavedClusterId();
          let clusterToSelect: any = null;
          
          if (savedId) {
            // Try to restore saved cluster
            clusterToSelect = clusters.find(c => c.id === savedId);
          }
          
          // If only one cluster exists, auto-select it
          if (!clusterToSelect && clusters.length === 1) {
            clusterToSelect = clusters[0];
          }
          
          if (clusterToSelect) {
            this.selectCluster(clusterToSelect);
          }
        }
      },
      error: (error) => {
        this.toastr.danger('加载集群列表失败', '错误');
        this.loading = false;
      },
    });
  }

  selectCluster(cluster: Cluster): void {
    this.clusterContext.setActiveCluster(cluster);
    this.toastr.success(`已切换到集群: ${cluster.name}`, '成功');
  }

  onClusterChange(cluster: Cluster): void {
    if (cluster) {
      this.selectCluster(cluster);
    }
  }

  compareById(c1: Cluster, c2: Cluster): boolean {
    return c1 && c2 ? c1.id === c2.id : c1 === c2;
  }

  goToClusterManagement(): void {
    this.router.navigate(['/pages/starrocks/clusters']);
  }

  refreshClusters(): void {
    this.loadClusters();
  }
}

