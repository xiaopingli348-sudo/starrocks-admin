import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { ApiService } from './api.service';
import { SystemFunction, CreateFunctionRequest, UpdateOrderRequest } from './system-function';

@Injectable({
  providedIn: 'root'
})
export class SystemFunctionService {

  constructor(private api: ApiService) {}

  // 获取集群的所有自定义功能
  getFunctions(clusterId: number): Observable<SystemFunction[]> {
    return this.api.get<SystemFunction[]>(`/clusters/${clusterId}/system-functions`);
  }

  // 创建自定义功能
  createFunction(clusterId: number, req: CreateFunctionRequest): Observable<SystemFunction> {
    return this.api.post<SystemFunction>(`/clusters/${clusterId}/system-functions`, req);
  }

  // 执行自定义功能的SQL
  executeFunction(clusterId: number, functionId: number): Observable<any> {
    return this.api.post<any>(`/clusters/${clusterId}/system-functions/${functionId}/execute`, {});
  }

  // 更新排序和分类顺序
  updateOrders(clusterId: number, orders: UpdateOrderRequest): Observable<void> {
    return this.api.put<void>(`/clusters/${clusterId}/system-functions/orders`, orders);
  }

  // 切换收藏状态
  toggleFavorite(clusterId: number, functionId: number): Observable<SystemFunction> {
    return this.api.put<SystemFunction>(`/clusters/${clusterId}/system-functions/${functionId}/favorite`, {});
  }

  // 删除自定义功能
  deleteFunction(clusterId: number, functionId: number): Observable<void> {
    return this.api.delete<void>(`/clusters/${clusterId}/system-functions/${functionId}`);
  }

  // 更新系统功能访问时间
  updateSystemFunctionAccessTime(functionName: string): Observable<void> {
    return this.api.put<void>(`/system-functions/${functionName}/access-time`, {});
  }

  // 删除分类
  deleteCategory(categoryName: string): Observable<void> {
    return this.api.delete<void>(`/system-functions/category/${encodeURIComponent(categoryName)}`);
  }

  // 更新功能
  updateFunction(clusterId: number, functionId: number, request: CreateFunctionRequest): Observable<SystemFunction> {
    return this.api.put<SystemFunction>(`/clusters/${clusterId}/system-functions/${functionId}`, request);
  }
}
