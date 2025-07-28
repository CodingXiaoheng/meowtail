import { Component, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet, RouterModule } from '@angular/router';
import { LoginComponent } from './login-component/login-component';
import { MatSidenavModule } from '@angular/material/sidenav';
import { MatButtonModule } from '@angular/material/button';
import { MatToolbarModule } from '@angular/material/toolbar';
import { MatIconModule } from '@angular/material/icon';
import { Observable, of } from 'rxjs';
import { UserInfo } from './user-info';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [
    CommonModule,
    RouterOutlet,
    LoginComponent,
    MatButtonModule,
    MatSidenavModule,
    MatToolbarModule,
    MatIconModule,
    RouterModule
  ],
  templateUrl: './app.html',
  styleUrls: ['./app.css']
})
export class App {
  protected readonly title = signal('meowtail-frontend');

   

  constructor(private userInfo: UserInfo) {}

  isLoggedIn = of(false);  

  ngOnInit(){
    this.isLoggedIn = this.userInfo.isLoggedIn();
  }

  onLogout(): void {
    localStorage.clear();
    window.location.reload();
  }

  /**
   * 检测当前是否已登录。
   * @returns Observable<boolean> - 已登录返回 true，否则返回 false
   */
  // isLogined(): Observable<boolean> {
  //   //return this.userInfo.isLoggedIn();
  //   return of(false);
  // }
}
