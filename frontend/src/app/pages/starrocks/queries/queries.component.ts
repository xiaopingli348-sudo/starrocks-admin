import { Component, OnInit, OnDestroy, TemplateRef, ViewChild } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { NbToastrService, NbDialogService } from '@nebular/theme';
import { LocalDataSource } from 'ng2-smart-table';
import { Subject } from 'rxjs';
import { takeUntil } from 'rxjs/operators';
import { NodeService, QueryHistoryItem, QueryExecuteResult } from '../../../@core/data/node.service';
import { ClusterContextService } from '../../../@core/data/cluster-context.service';
import { Cluster } from '../../../@core/data/cluster.service';
import { ErrorHandler } from '../../../@core/utils/error-handler';

@Component({
  selector: 'ngx-queries',
  templateUrl: './queries.component.html',
  styleUrls: ['./queries.component.scss'],
})
export class QueriesComponent implements OnInit, OnDestroy {
  // Data sources
  runningSource: LocalDataSource = new LocalDataSource();
  historySource: LocalDataSource = new LocalDataSource();
  realtimeResultSource: LocalDataSource = new LocalDataSource();
  profileSource: LocalDataSource = new LocalDataSource();
  
  // Expose Math to template
  Math = Math;
  
  // State
  clusterId: number;
  activeCluster: Cluster | null = null;
  loading = true;
  selectedTab = 'realtime'; // 'realtime', 'running', 'history', or 'profiles'
  autoRefresh = false; // Default: disabled
  refreshInterval: any;
  selectedRefreshInterval = 5; // Default 5 seconds
  refreshIntervalOptions = [
    { value: 3, label: '3秒' },
    { value: 5, label: '5秒' },
    { value: 10, label: '10秒' },
    { value: 30, label: '30秒' },
    { value: 60, label: '1分钟' },
  ];
  private destroy$ = new Subject<void>();

  // Profile dialog
  currentProfile: any = null;
  currentProfileDetail: string = '';
  @ViewChild('profileDialog') profileDialogTemplate: TemplateRef<any>;
  @ViewChild('profileDetailDialog') profileDetailDialogTemplate: TemplateRef<any>;

  // Real-time query state
  sqlInput: string = '';
  queryResult: QueryExecuteResult | null = null;
  resultSettings: any = null;
  executing: boolean = false;
  executionTime: number = 0;
  rowCount: number = 0;
  queryLimit: number = 1000; // Default limit for query results
  limitOptions = [
    { value: 100, label: '100 行' },
    { value: 500, label: '500 行' },
    { value: 1000, label: '1000 行' },
    { value: 5000, label: '5000 行' },
    { value: 10000, label: '10000 行' },
  ];

  // History search filters
  searchKeyword: string = '';
  searchStartTime: string = '';
  searchEndTime: string = '';

  // Pagination state for history
  historyPageSize: number = 10;
  historyCurrentPage: number = 1;
  historyTotalCount: number = 0;
  
  // Running queries settings
  runningSettings = {
    mode: 'external',
    hideSubHeader: false, // Enable search
    noDataMessage: '当前没有运行中的查询',
    actions: false,
    pager: {
      display: true,
      perPage: 20,
    },
    columns: {
      QueryId: { title: 'Query ID', type: 'string' },
      User: { title: '用户', type: 'string', width: '10%' },
      Database: { title: '数据库', type: 'string', width: '10%' },
      ExecTime: { title: '执行时间', type: 'string', width: '10%' },
      Sql: { title: 'SQL', type: 'string' },
    },
  };

  // History queries settings with Profile button
  historySettings = {
    mode: 'external',
    hideSubHeader: false, // Enable search
    noDataMessage: '暂无历史查询记录',
    actions: {
      add: false,
      edit: true,
      delete: false,
      position: 'right',
    },
    edit: {
      editButtonContent: '<i class="nb-search"></i>',
    },
    pager: {
      display: false, // Disable ng2-smart-table's built-in pagination (we'll use custom pagination)
    },
    columns: {
      query_id: { title: 'Query ID', type: 'string' },
      user: { title: '用户', type: 'string', width: '8%' },
      default_db: { title: '数据库', type: 'string', width: '8%' },
      query_type: { title: '类型', type: 'string', width: '8%' },
      query_state: { title: '状态', type: 'string', width: '8%' },
      start_time: { title: '开始时间', type: 'string', width: '12%' },
      total_ms: { title: '耗时(ms)', type: 'number', width: '8%' },
      sql_statement: { title: 'SQL', type: 'string' },
    },
  };

  // Profile management settings
  profileSettings = {
    mode: 'external',
    hideSubHeader: false, // Enable search
    noDataMessage: '暂无Profile记录',
    actions: {
      add: false,
      edit: true,
      delete: false,
      position: 'right',
    },
    edit: {
      editButtonContent: '<i class="nb-search"></i>',
    },
    pager: {
      display: true,
      perPage: 20,
    },
    columns: {
      QueryId: { title: 'Query ID', type: 'string', width: '25%' },
      StartTime: { title: '开始时间', type: 'string', width: '15%' },
      Time: { title: '执行时间', type: 'string', width: '10%' },
      State: {
        title: '状态',
        type: 'html',
        width: '10%',
        valuePrepareFunction: (value: string) => {
          const status = value === 'Finished' ? 'success' : 'warning';
          return `<span class="badge badge-${status}">${value}</span>`;
        },
      },
      Statement: { title: 'SQL语句', type: 'string', width: '40%' },
    },
  };

  constructor(
    private nodeService: NodeService,
    private route: ActivatedRoute,
    private toastrService: NbToastrService,
    private clusterContext: ClusterContextService,
    private dialogService: NbDialogService,
  ) {
    // Try to get clusterId from route first (for direct navigation)
    const routeClusterId = parseInt(this.route.snapshot.paramMap.get('clusterId') || '0', 10);
    this.clusterId = routeClusterId;
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
            this.clusterId = newClusterId;
            // Reset pagination when cluster changes
            this.historyCurrentPage = 1;
            // Only load if not on realtime tab
            if (this.selectedTab !== 'realtime') {
              this.loadCurrentTab();
            } else {
              this.loading = false;
            }
          }
        }
      });

    // Load queries if clusterId is already set from route
    if (this.clusterId && this.clusterId > 0) {
      // Only load if not on realtime tab
      if (this.selectedTab !== 'realtime') {
        this.loadCurrentTab();
      } else {
        this.loading = false;
      }
    } else if (!this.clusterContext.hasActiveCluster()) {
      this.toastrService.warning('请先在集群概览页面激活一个集群', '提示');
      this.loading = false;
    }
  }

  ngOnDestroy(): void {
    this.stopAutoRefresh();
    this.destroy$.next();
    this.destroy$.complete();
  }

  // Tab switching
  selectTab(tab: string): void {
    this.selectedTab = tab;
    this.loadCurrentTab();
  }

  // Auto refresh methods
  toggleAutoRefresh(): void {
    this.autoRefresh = !this.autoRefresh;
    if (this.autoRefresh) {
      this.startAutoRefresh();
    } else {
      this.stopAutoRefresh();
    }
  }

  onRefreshIntervalChange(interval: number): void {
    this.selectedRefreshInterval = interval;
    if (this.autoRefresh) {
      // Restart with new interval
      this.stopAutoRefresh();
      this.startAutoRefresh();
    }
  }

  startAutoRefresh(): void {
    this.stopAutoRefresh(); // Clear any existing interval
    this.refreshInterval = setInterval(() => {
      this.loadCurrentTab();
    }, this.selectedRefreshInterval * 1000);
  }

  stopAutoRefresh(): void {
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval);
      this.refreshInterval = null;
    }
  }

  // Load data based on current tab
  loadCurrentTab(): void {
    if (this.selectedTab === 'running') {
      this.loadRunningQueries();
    } else if (this.selectedTab === 'history') {
      this.loadHistoryQueries();
    } else if (this.selectedTab === 'profiles') {
      this.loadProfiles();
    } else {
      // realtime tab doesn't need auto-loading
      this.loading = false;
    }
  }

  // Load running queries
  loadRunningQueries(): void {
    this.loading = true;
    this.nodeService.listQueries().subscribe({
      next: (queries) => {
        this.runningSource.load(queries);
        this.loading = false;
      },
      error: (error) => {
        this.toastrService.danger(ErrorHandler.extractErrorMessage(error), '加载失败');
        this.loading = false;
      },
    });
  }

  // Load history queries with server-side pagination
  loadHistoryQueries(): void {
    const offset = (this.historyCurrentPage - 1) * this.historyPageSize;
    
    this.loading = true;
    
    this.nodeService.listQueryHistory(this.historyPageSize, offset).subscribe({
      next: (response) => {
        
        // Update total count for pagination
        this.historyTotalCount = response.total;
        
        // Load data into table
        this.historySource.load(response.data);
        this.loading = false;
      },
      error: (error) => {
        this.toastrService.danger(ErrorHandler.extractErrorMessage(error), '加载失败');
        this.historySource.load([]);
        this.historyTotalCount = 0;
        this.loading = false;
      },
      complete: () => {
        // Ensure loading is always set to false
        this.loading = false;
      }
    });
  }

  // Calculate total pages
  get historyTotalPages(): number {
    return Math.ceil(this.historyTotalCount / this.historyPageSize);
  }

  // Handle page change
  onHistoryPageChange(page: number): void {
    if (page < 1 || page > this.historyTotalPages) {
      return;
    }
    this.historyCurrentPage = page;
    this.loadHistoryQueries();
  }

  // Handle page size change
  onHistoryPageSizeChange(size: number): void {
    this.historyPageSize = size;
    this.historyCurrentPage = 1; // Reset to first page
    this.loadHistoryQueries();
  }

  // Handle edit action (View Profile)
  onEditProfile(event: any): void {
    const query: QueryHistoryItem = event.data;
    this.viewProfile(query.query_id);
  }

  // View query profile
  viewProfile(queryId: string): void {
    this.nodeService.getQueryProfile(queryId).subscribe({
      next: (profile) => {
        this.currentProfile = profile;
        // Open profile dialog
        this.dialogService.open(this.profileDialogTemplate, {
          context: { profile },
        });
      },
      error: (error) => {
        this.toastrService.danger(ErrorHandler.extractErrorMessage(error), '加载失败');
      },
    });
  }

  // Real-time query methods
  executeSQL(): void {
    if (!this.sqlInput || this.sqlInput.trim() === '') {
      this.toastrService.warning('请输入SQL语句', '提示');
      return;
    }

    this.executing = true;
    this.queryResult = null;
    this.resultSettings = null;

    this.nodeService.executeSQL(this.sqlInput.trim(), this.queryLimit).subscribe({
      next: (result) => {
        this.queryResult = result;
        this.executionTime = result.execution_time_ms;
        this.rowCount = result.row_count;

        // Build dynamic table settings
        this.buildResultSettings(result);

        // Convert rows to objects for ng2-smart-table
        const dataRows = result.rows.map(row => {
          const obj: any = {};
          result.columns.forEach((col, idx) => {
            obj[col] = row[idx];
          });
          return obj;
        });

        this.realtimeResultSource.load(dataRows);
        this.executing = false;
        this.toastrService.success(`查询成功，返回 ${result.row_count} 行`, '成功');
      },
      error: (error) => {
        this.executing = false;
        this.toastrService.danger(ErrorHandler.extractErrorMessage(error), '执行失败');
      },
    });
  }

  buildResultSettings(result: QueryExecuteResult): void {
    const columns: any = {};
    result.columns.forEach(col => {
      columns[col] = { title: col, type: 'string' };
    });

    this.resultSettings = {
      mode: 'external',
      hideSubHeader: false, // Enable search
      noDataMessage: '无数据',
      actions: false,
      pager: {
        display: true,
        perPage: 50,
      },
      columns: columns,
    };
  }

  clearSQL(): void {
    this.sqlInput = '';
    this.queryResult = null;
    this.resultSettings = null;
    this.executionTime = 0;
    this.rowCount = 0;
  }

  formatSQL(): void {
    if (!this.sqlInput) {
      return;
    }
    // Simple SQL formatting (basic implementation)
    let formatted = this.sqlInput.trim();
    // Add line breaks before major keywords
    formatted = formatted.replace(/\s+(SELECT|FROM|WHERE|GROUP BY|ORDER BY|LIMIT|JOIN|LEFT JOIN|RIGHT JOIN|INNER JOIN)/gi, '\n$1');
    // Capitalize keywords
    formatted = formatted.replace(/\b(SELECT|FROM|WHERE|GROUP BY|ORDER BY|LIMIT|AS|ON|AND|OR|JOIN|LEFT|RIGHT|INNER|OUTER|DESC|ASC)\b/gi, match => match.toUpperCase());
    this.sqlInput = formatted;
  }

  // Search history methods
  searchHistory(): void {
    if (!this.clusterId) {
      return;
    }
    
    // Reload history with current filters
    this.loadHistoryQueries();
  }

  applyHistoryFilters(queries: QueryHistoryItem[]): QueryHistoryItem[] {
    let filtered = queries;

    // Filter by keyword (search in query_id and sql_statement)
    if (this.searchKeyword && this.searchKeyword.trim() !== '') {
      const keyword = this.searchKeyword.toLowerCase();
      filtered = filtered.filter(q => 
        q.query_id.toLowerCase().includes(keyword) || 
        q.sql_statement.toLowerCase().includes(keyword) ||
        q.user.toLowerCase().includes(keyword)
      );
    }

    // Filter by start time
    if (this.searchStartTime) {
      filtered = filtered.filter(q => q.start_time >= this.searchStartTime);
    }

    // Filter by end time
    if (this.searchEndTime) {
      filtered = filtered.filter(q => q.start_time <= this.searchEndTime);
    }

    return filtered;
  }

  // Export results to CSV
  exportResults(): void {
    if (!this.queryResult || !this.queryResult.rows || this.queryResult.rows.length === 0) {
      this.toastrService.warning('没有数据可导出', '提示');
      return;
    }

    try {
      // Build CSV content
      const columns = this.queryResult.columns;
      const rows = this.queryResult.rows;

      // CSV header
      let csvContent = columns.map(col => this.escapeCSV(col)).join(',') + '\n';

      // CSV rows
      rows.forEach(row => {
        csvContent += row.map(cell => this.escapeCSV(cell)).join(',') + '\n';
      });

      // Create blob and download
      const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
      const link = document.createElement('a');
      const url = URL.createObjectURL(blob);
      
      link.setAttribute('href', url);
      link.setAttribute('download', `query_result_${new Date().getTime()}.csv`);
      link.style.visibility = 'hidden';
      
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);

      this.toastrService.success('导出成功', '成功');
    } catch (error) {
      console.error('Export error:', error);
      this.toastrService.danger(ErrorHandler.extractErrorMessage(error), '导出失败');
    }
  }

  // Escape CSV special characters
  private escapeCSV(value: string): string {
    if (value === null || value === undefined) {
      return '';
    }
    const stringValue = String(value);
    if (stringValue.includes(',') || stringValue.includes('"') || stringValue.includes('\n')) {
      return '"' + stringValue.replace(/"/g, '""') + '"';
    }
    return stringValue;
  }

  // Load profiles
  loadProfiles(): void {
    this.loading = true;
    this.nodeService.listProfiles().subscribe(
      data => {
        this.profileSource.load(data);
        this.loading = false;
      },
      error => {
        this.toastrService.danger(ErrorHandler.extractErrorMessage(error), '加载失败');
        this.loading = false;
      }
    );
  }

  // Handle profile edit action (view profile)
  onProfileEdit(event: any): void {
    this.viewProfileDetail(event.data.QueryId);
  }

  // View profile detail from profile list
  viewProfileDetail(queryId: string): void {
    this.nodeService.getProfile(queryId).subscribe(
      data => {
        this.currentProfileDetail = data.profile_content;
        this.dialogService.open(this.profileDetailDialogTemplate, {
          context: { profile: this.currentProfileDetail },
          hasBackdrop: true,
          closeOnBackdropClick: true,
          closeOnEsc: true,
          dialogClass: 'modal-lg', // Use Bootstrap's large modal class
        });
      },
      error => {
        this.toastrService.danger(ErrorHandler.extractErrorMessage(error), '加载失败');
      }
    );
  }
}
