import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { ApiService } from './api.service';

export interface MetricsSummary {
  // Query metrics
  qps: number;
  rps: number;
  query_total: number;
  query_success: number;
  query_err: number;
  query_timeout: number;
  query_err_rate: number;
  query_latency_p50: number;
  query_latency_p95: number;
  query_latency_p99: number;

  // FE system metrics
  jvm_heap_total: number;
  jvm_heap_used: number;
  jvm_heap_usage_pct: number;
  jvm_thread_count: number;

  // Backend aggregate metrics
  backend_total: number;
  backend_alive: number;
  tablet_count: number;
  disk_total_bytes: number;
  disk_used_bytes: number;
  disk_usage_pct: number;
  avg_cpu_usage_pct: number;
  avg_mem_usage_pct: number;
  total_running_queries: number;

  // Storage metrics
  max_compaction_score: number;

  // Transaction metrics
  txn_begin: number;
  txn_success: number;
  txn_failed: number;

  // Load metrics
  load_finished: number;
  routine_load_rows: number;
}

export interface RuntimeInfo {
  fe_node: string;
  total_mem: number;
  free_mem: number;
  thread_cnt: number;
}

@Injectable({
  providedIn: 'root',
})
export class MonitorService {
  constructor(private api: ApiService) {}

  getMetricsSummary(clusterId: number): Observable<MetricsSummary> {
    return this.api.get<MetricsSummary>(`/clusters/${clusterId}/metrics/summary`);
  }

  getRuntimeInfo(clusterId: number): Observable<RuntimeInfo> {
    return this.api.get<RuntimeInfo>(`/clusters/${clusterId}/system/runtime_info`);
  }
}

