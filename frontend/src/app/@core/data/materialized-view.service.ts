import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { ApiService } from './api.service';

export interface MaterializedView {
  id: string;
  name: string;
  database_name: string;
  refresh_type: string; // ROLLUP/MANUAL/ASYNC/INCREMENTAL
  is_active: boolean;
  partition_type?: string;
  task_id?: string;
  task_name?: string;
  last_refresh_start_time?: string;
  last_refresh_finished_time?: string;
  last_refresh_duration?: string;
  last_refresh_state?: string; // SUCCESS/RUNNING/FAILED/PENDING
  last_refresh_force_refresh?: boolean;
  last_refresh_start_partition?: string;
  last_refresh_end_partition?: string;
  last_refresh_base_refresh_partitions?: string;
  last_refresh_mv_refresh_partitions?: string;
  last_refresh_error_code?: string;
  last_refresh_error_message?: string;
  rows?: number;
  text: string;
}

export interface CreateMaterializedViewRequest {
  sql: string;
}

export interface RefreshMaterializedViewRequest {
  partition_start?: string;
  partition_end?: string;
  force: boolean;
  mode: string; // SYNC/ASYNC
}

export interface AlterMaterializedViewRequest {
  alter_clause: string;
}

export interface MaterializedViewDDL {
  mv_name: string;
  ddl: string;
}

@Injectable()
export class MaterializedViewService {
  constructor(private api: ApiService) {}

  getMaterializedViews(clusterId: number, database?: string): Observable<MaterializedView[]> {
    const params = database ? { database } : {};
    return this.api.get<MaterializedView[]>(
      `/clusters/${clusterId}/materialized_views`,
      params
    );
  }

  getMaterializedView(clusterId: number, mvName: string): Observable<MaterializedView> {
    return this.api.get<MaterializedView>(
      `/clusters/${clusterId}/materialized_views/${mvName}`
    );
  }

  getMaterializedViewDDL(clusterId: number, mvName: string): Observable<MaterializedViewDDL> {
    return this.api.get<MaterializedViewDDL>(
      `/clusters/${clusterId}/materialized_views/${mvName}/ddl`
    );
  }

  createMaterializedView(
    clusterId: number,
    request: CreateMaterializedViewRequest
  ): Observable<any> {
    return this.api.post(`/clusters/${clusterId}/materialized_views`, request);
  }

  deleteMaterializedView(
    clusterId: number,
    mvName: string,
    ifExists: boolean = true
  ): Observable<any> {
    const params = { if_exists: ifExists };
    return this.api.delete(
      `/clusters/${clusterId}/materialized_views/${mvName}?if_exists=${ifExists}`
    );
  }

  refreshMaterializedView(
    clusterId: number,
    mvName: string,
    request: RefreshMaterializedViewRequest
  ): Observable<any> {
    return this.api.post(
      `/clusters/${clusterId}/materialized_views/${mvName}/refresh`,
      request
    );
  }

  cancelRefreshMaterializedView(
    clusterId: number,
    mvName: string,
    force: boolean = false
  ): Observable<any> {
    return this.api.post(
      `/clusters/${clusterId}/materialized_views/${mvName}/cancel?force=${force}`,
      {}
    );
  }

  alterMaterializedView(
    clusterId: number,
    mvName: string,
    request: AlterMaterializedViewRequest
  ): Observable<any> {
    return this.api.put(
      `/clusters/${clusterId}/materialized_views/${mvName}`,
      request
    );
  }
}

