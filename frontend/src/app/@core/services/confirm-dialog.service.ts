import { Injectable } from '@angular/core';
import { NbDialogService } from '@nebular/theme';
import { Observable } from 'rxjs';
import { ConfirmDialogComponent, ConfirmDialogData } from '../components/confirm-dialog/confirm-dialog.component';

@Injectable({
  providedIn: 'root'
})
export class ConfirmDialogService {
  constructor(private dialogService: NbDialogService) {}

  confirm(data: ConfirmDialogData): Observable<boolean> {
    return this.dialogService.open(ConfirmDialogComponent, {
      context: { data },
      hasBackdrop: true,
      closeOnBackdropClick: false,
      closeOnEsc: true,
      autoFocus: true,
      dialogClass: 'confirm-dialog'
    }).onClose;
  }

  confirmDelete(itemName: string, additionalWarning?: string): Observable<boolean> {
    const message = additionalWarning 
      ? `确定要删除 "${itemName}" 吗？\n\n${additionalWarning}`
      : `确定要删除 "${itemName}" 吗？`;

    return this.confirm({
      title: '确认删除',
      message: message,
      confirmText: '删除',
      cancelText: '取消',
      type: 'danger'
    });
  }
}
