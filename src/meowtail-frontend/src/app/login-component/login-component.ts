// login-component.ts
import { Component } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { HttpClientModule, HttpClient } from '@angular/common/http';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatCardModule } from '@angular/material/card';

@Component({
  selector: 'app-login-component',
  standalone: true,
  imports: [
    MatIconModule,
    MatInputModule,
    MatButtonModule,
    MatFormFieldModule,
    MatCardModule,
    FormsModule,
    HttpClientModule
  ],
  templateUrl: './login-component.html',
  styleUrls: ['./login-component.css'],
})
export class LoginComponent {
  username = '';
  password = '';
  hide = true;

  constructor(private http: HttpClient) {}

  onSubmit() {
    this.http.post<{ token: string }>('/login', {
      username: this.username,
      password: this.password
    })
    .subscribe({
      next: res => {
        // 登录成功
        localStorage.setItem('token', res.token);
        // 刷新页面
        window.location.reload();
      },
      error: err => {
        // 登录失败
        console.error('登录失败', err); // 保留控制台错误，方便调试
        alert('登录失败，请检查您的用户名和密码！'); // 弹出提示
      }
    });
  }
}