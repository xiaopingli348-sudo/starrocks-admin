import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';
import { Cluster } from './cluster.service';

/**
 * Global cluster context service
 * Manages the currently active cluster across the application
 */
@Injectable({
  providedIn: 'root',
})
export class ClusterContextService {
  private readonly STORAGE_KEY = 'active_cluster_id';
  
  // Current active cluster
  private activeClusterSubject: BehaviorSubject<Cluster | null>;
  public activeCluster$: Observable<Cluster | null>;
  
  constructor() {
    // Try to restore active cluster from localStorage
    const savedClusterId = this.getSavedClusterId();
    console.log('[ClusterContext] Constructor - Saved clusterId:', savedClusterId);
    this.activeClusterSubject = new BehaviorSubject<Cluster | null>(null);
    this.activeCluster$ = this.activeClusterSubject.asObservable();
  }
  
  /**
   * Set the active cluster
   */
  setActiveCluster(cluster: Cluster | null): void {
    console.log('[ClusterContext] Setting active cluster:', cluster);
    this.activeClusterSubject.next(cluster);
    
    if (cluster) {
      localStorage.setItem(this.STORAGE_KEY, cluster.id.toString());
      console.log('[ClusterContext] Saved clusterId to localStorage:', cluster.id);
    } else {
      localStorage.removeItem(this.STORAGE_KEY);
      console.log('[ClusterContext] Removed clusterId from localStorage');
    }
  }
  
  /**
   * Get the current active cluster
   */
  getActiveCluster(): Cluster | null {
    return this.activeClusterSubject.value;
  }
  
  /**
   * Get the active cluster ID
   */
  getActiveClusterId(): number | null {
    const cluster = this.activeClusterSubject.value;
    return cluster ? cluster.id : null;
  }
  
  /**
   * Get saved cluster ID from localStorage
   */
  getSavedClusterId(): number | null {
    const saved = localStorage.getItem(this.STORAGE_KEY);
    return saved ? parseInt(saved, 10) : null;
  }
  
  /**
   * Clear active cluster
   */
  clearActiveCluster(): void {
    this.setActiveCluster(null);
  }
  
  /**
   * Check if a cluster is active
   */
  hasActiveCluster(): boolean {
    return this.activeClusterSubject.value !== null;
  }
}

