import { Component, OnInit } from '@angular/core';
import { Router, NavigationEnd } from '@angular/router';
import { NbMenuService } from '@nebular/theme';
import { filter, map } from 'rxjs/operators';

import { MENU_ITEMS } from './pages-menu';
import { AuthService } from '../@core/data/auth.service';
import { TabService } from '../@core/services/tab.service';

@Component({
  selector: 'ngx-pages',
  styleUrls: ['pages.component.scss'],
  template: `
    <ngx-one-column-layout>
      <nb-menu [items]="menu" tag="menu" (itemClick)="onMenuClick($event)"></nb-menu>
      <router-outlet></router-outlet>
    </ngx-one-column-layout>
  `,
})
export class PagesComponent implements OnInit {
  menu = MENU_ITEMS;

  constructor(
    private menuService: NbMenuService,
    private authService: AuthService,
    private router: Router,
    private tabService: TabService
  ) {}

  ngOnInit() {
    // Listen to menu item clicks
    this.menuService.onItemClick()
      .pipe(
        filter(({ tag }) => tag === 'menu'),
        map(({ item }) => item)
      )
      .subscribe(item => {
        if (item.title === '退出登录') {
          this.authService.logout();
        }
      });

    // Listen to route changes and add tabs
    // Only handle route changes when navigation is triggered by menu or direct URL
    this.router.events
      .pipe(filter(event => event instanceof NavigationEnd))
      .subscribe((event: NavigationEnd) => {
        // Skip if this is triggered by tab switching
        // Check if the URL matches any existing active tab
        const activeTab = this.tabService.getActiveTab();
        if (activeTab && activeTab.url === event.url) {
          // This navigation is from tab switching, skip adding new tab
          return;
        }
        this.handleRouteChange(event.url);
      });
  }

  /**
   * 处理路由变化，自动添加Tab
   */
  private handleRouteChange(url: string): void {
    // 跳过登录页面
    if (url.includes('/auth/')) {
      return;
    }

    // 查找对应的菜单项
    const menuItem = this.findMenuItemByUrl(url);
    if (menuItem) {
      const tabId = this.generateTabId(menuItem.title);
      // 路由变化时不再触发导航（因为已经在目标路由了）
      this.tabService.addTab({
        id: tabId,
        title: menuItem.title,
        url: url,
        closable: true,
        pinned: false
      }, false);
    } else {
      // 如果没有找到对应的菜单项，尝试从URL推断标题
      const title = this.inferTitleFromUrl(url);
      if (title) {
        const tabId = this.generateTabId(title);
        // 路由变化时不再触发导航（因为已经在目标路由了）
        this.tabService.addTab({
          id: tabId,
          title: title,
          url: url,
          closable: true,
          pinned: false
        }, false);
      }
    }
  }

  /**
   * 从URL推断标题
   */
  private inferTitleFromUrl(url: string): string | null {
    const urlSegments = url.split('/').filter(segment => segment);
    
    // 处理StarRocks相关路由
    if (urlSegments.includes('starrocks')) {
      const lastSegment = urlSegments[urlSegments.length - 1];
      
      // 映射URL段到中文标题
      const titleMap: { [key: string]: string } = {
        'dashboard': '集群列表',
        'overview': '集群概览',
        'frontends': 'Frontend 节点',
        'backends': 'Backend 节点',
        'execution': '实时查询',
        'profiles': 'Profiles',
        'audit-logs': '审计日志',
        'materialized-views': '物化视图',
        'system': '功能卡片',
        'sessions': '会话管理',
        'variables': '变量管理',
        'clusters': '集群管理',
        'new': '新建集群',
        'edit': '编辑集群'
      };
      
      return titleMap[lastSegment] || lastSegment;
    }
    
    return null;
  }

  /**
   * 根据URL查找菜单项
   */
  private findMenuItemByUrl(url: string): any {
    const findInMenu = (items: any[]): any => {
      for (const item of items) {
        if (item.link === url) {
          return item;
        }
        if (item.children) {
          const found = findInMenu(item.children);
          if (found) return found;
        }
      }
      return null;
    };

    return findInMenu(MENU_ITEMS);
  }

  /**
   * 生成Tab ID（基于标题，确保每个页面有唯一ID）
   */
  private generateTabId(title: string): string {
    // 使用标题生成固定ID，每个页面名称对应唯一ID
    return 'tab_' + title.replace(/[^a-zA-Z0-9\u4e00-\u9fa5]/g, '_');
  }

  onMenuClick(event: any) {
    if (event.item.title === '退出登录') {
      event.event.preventDefault();
      this.authService.logout();
    }
  }
}
