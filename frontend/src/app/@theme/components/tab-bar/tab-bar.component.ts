import { Component, OnInit, OnDestroy, ChangeDetectionStrategy, ChangeDetectorRef } from '@angular/core';
import { Subject } from 'rxjs';
import { takeUntil } from 'rxjs/operators';
import { TabService, TabItem } from '../../../@core/services/tab.service';

@Component({
  selector: 'ngx-tab-bar',
  templateUrl: './tab-bar.component.html',
  styleUrls: ['./tab-bar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class TabBarComponent implements OnInit, OnDestroy {
  tabs: TabItem[] = [];
  private destroy$ = new Subject<void>();

  constructor(
    private tabService: TabService,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.tabService.tabs$
      .pipe(takeUntil(this.destroy$))
      .subscribe(tabs => {
        this.tabs = tabs;
        // Manually trigger change detection for OnPush strategy
        this.cdr.markForCheck();
      });
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  /**
   * 点击Tab激活
   */
  onTabClick(tab: TabItem): void {
    this.tabService.activateTab(tab.id);
  }

  /**
   * 关闭Tab
   */
  onCloseTab(event: Event, tabId: string): void {
    event.stopPropagation();
    this.tabService.closeTab(tabId);
  }

  /**
   * TrackBy函数优化渲染性能
   */
  trackByTabId(index: number, tab: TabItem): string {
    return tab.id;
  }
}
