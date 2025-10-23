import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { NbMenuService } from '@nebular/theme';
import { filter, map } from 'rxjs/operators';

import { MENU_ITEMS } from './pages-menu';
import { AuthService } from '../@core/data/auth.service';

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
    private router: Router
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
  }

  onMenuClick(event: any) {
    if (event.item.title === '退出登录') {
      event.event.preventDefault();
      this.authService.logout();
    }
  }
}
