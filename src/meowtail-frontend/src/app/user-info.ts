import { Injectable, Inject, PLATFORM_ID } from '@angular/core';
import { HttpClient, HttpResponse } from '@angular/common/http';
import { Observable, of } from 'rxjs';
import { map, catchError } from 'rxjs/operators';
import { isPlatformBrowser } from '@angular/common'; 

@Injectable({
  providedIn: 'root'
})
export class UserInfo {

  private readonly CHECK_URL = '/api/logined';

  constructor(private http: HttpClient,
    @Inject(PLATFORM_ID) private platformId: Object
  ) {}

  /**
   * 检测当前是否已登录。
   * @returns Observable<boolean> - 已登录返回 true，否则返回 false
   */
  isLoggedIn(): Observable<boolean> {
    if (isPlatformBrowser(this.platformId)){
          return this.http
      .get(this.CHECK_URL, { observe: 'response' })
      .pipe(
        map((res: HttpResponse<any>) => res.status === 200),
        catchError(() => of(false))
      );
    }else{
      return of(false);
    }

  }
}
