import { Component, Inject } from '@angular/core';
import { NbDialogRef } from '@nebular/theme';

export interface ConfirmDialogData {
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  type?: 'primary' | 'success' | 'warning' | 'danger' | 'info';
}

@Component({
  selector: 'ngx-confirm-dialog',
  template: `
    <nb-card>
      <nb-card-header>
        <div class="d-flex align-items-center">
          <nb-icon 
            [icon]="getIcon()" 
            [status]="data.type || 'warning'"
            class="me-2">
          </nb-icon>
          <h6 class="mb-0">{{ data.title }}</h6>
        </div>
      </nb-card-header>
      <nb-card-body>
        <p class="mb-0">{{ data.message }}</p>
      </nb-card-body>
      <nb-card-footer>
        <div class="d-flex justify-content-end gap-2">
          <button 
            nbButton 
            status="basic" 
            size="small"
            (click)="onCancel()">
            {{ data.cancelText || '取消' }}
          </button>
          <button 
            nbButton 
            [status]="data.type || 'warning'" 
            size="small"
            (click)="onConfirm()">
            {{ data.confirmText || '确定' }}
          </button>
        </div>
      </nb-card-footer>
    </nb-card>
  `,
  styles: [`
    nb-card {
      margin: 0;
      min-width: 300px;
    }
    
    .gap-2 {
      gap: 0.5rem;
    }
  `]
})
export class ConfirmDialogComponent {
  data: ConfirmDialogData;

  constructor(
    @Inject(NbDialogRef) protected dialogRef: NbDialogRef<ConfirmDialogComponent>,
    @Inject('data') context: { data: ConfirmDialogData }
  ) {
    this.data = context.data;
  }

  getIcon(): string {
    switch (this.data.type) {
      case 'danger':
        return 'trash-2-outline';
      case 'warning':
        return 'alert-triangle-outline';
      case 'success':
        return 'checkmark-circle-2-outline';
      case 'info':
        return 'info-outline';
      default:
        return 'alert-triangle-outline';
    }
  }

  onConfirm(): void {
    this.dialogRef.close(true);
  }

  onCancel(): void {
    this.dialogRef.close(false);
  }
}
