import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { DashboardComponent } from './dashboard/dashboard.component';
import { ClusterListComponent } from './clusters/cluster-list/cluster-list.component';
import { ClusterFormComponent } from './clusters/cluster-form/cluster-form.component';
import { ClusterDetailComponent } from './clusters/cluster-detail/cluster-detail.component';
import { BackendsComponent } from './backends/backends.component';
import { FrontendsComponent } from './frontends/frontends.component';
import { MaterializedViewsComponent } from './materialized-views/materialized-views.component';
import { QueriesComponent } from './queries/queries.component';
import { MonitorComponent } from './monitor/monitor.component';
import { SessionsComponent } from './sessions/sessions.component';
import { VariablesComponent } from './variables/variables.component';
import { SystemManagementComponent } from './system-management/system-management.component';

const routes: Routes = [
  {
    path: '',
    redirectTo: 'dashboard',
    pathMatch: 'full',
  },
  {
    path: 'dashboard',
    component: DashboardComponent,
  },
  {
    path: 'clusters',
    children: [
      {
        path: '',
        component: ClusterListComponent,
      },
      {
        path: 'new',
        component: ClusterFormComponent,
      },
      {
        path: ':id',
        component: ClusterDetailComponent,
      },
      {
        path: ':id/edit',
        component: ClusterFormComponent,
      },
    ],
  },
  {
    path: 'backends',
    component: BackendsComponent,
  },
  {
    path: 'frontends',
    component: FrontendsComponent,
  },
  {
    path: 'materialized-views',
    component: MaterializedViewsComponent,
  },
  {
    path: 'queries',
    component: QueriesComponent,
  },
  {
    path: 'sessions',
    component: SessionsComponent,
  },
  {
    path: 'variables',
    component: VariablesComponent,
  },
  {
    path: 'system',
    component: SystemManagementComponent,
  },
  {
    path: 'monitor',
    component: MonitorComponent,
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class StarRocksRoutingModule {}

