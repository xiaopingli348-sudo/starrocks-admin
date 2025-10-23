import { Component, OnInit } from '@angular/core';
import { Router, ActivatedRoute } from '@angular/router';
import { NbToastrService } from '@nebular/theme';
import { AuthService } from '../../@core/data/auth.service';

@Component({
  selector: 'ngx-login',
  templateUrl: './login.component.html',
  styleUrls: ['./login.component.scss']
})
export class LoginComponent implements OnInit {
  submitted = false;
  user = {
    username: '',
    password: ''
  };
  errors: string[] = [];
  messages: string[] = [];
  showMessages = false;
  returnUrl: string;

  constructor(
    protected router: Router,
    private route: ActivatedRoute,
    private authService: AuthService,
    private toastrService: NbToastrService
  ) {}

  ngOnInit() {
    // Get return URL from route parameters or default to dashboard
    this.returnUrl = this.route.snapshot.queryParams['returnUrl'] || '/pages/starrocks/dashboard';
    
    // If already logged in, redirect to return URL
    if (this.authService.isAuthenticated()) {
      this.router.navigate([this.returnUrl]);
    }
  }

  login(): void {
    this.errors = [];
    this.messages = [];
    this.submitted = true;

    if (!this.user.username || !this.user.password) {
      this.errors.push('Username and password are required!');
      this.submitted = false;
      return;
    }

    this.authService.login(this.user).subscribe({
      next: (response) => {
        this.submitted = false;
        this.messages = ['Successfully logged in!'];
        this.showMessages = true;
        this.toastrService.success('Welcome back!', 'Login Successful');
        // Navigate to return URL or dashboard after short delay
        setTimeout(() => {
          this.router.navigate([this.returnUrl]);
        }, 500);
      },
      error: (error) => {
        this.submitted = false;
        this.errors = [error.error?.message || 'Login failed. Please check your credentials.'];
        this.showMessages = true;
        this.toastrService.danger(this.errors[0], 'Login Failed');
      }
    });
  }
}
