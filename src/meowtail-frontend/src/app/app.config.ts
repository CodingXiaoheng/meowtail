import { ApplicationConfig, provideBrowserGlobalErrorListeners, provideZoneChangeDetection } from '@angular/core';
import { provideRouter } from '@angular/router';
import { importProvidersFrom } from '@angular/core';
import { routes } from './app.routes';
import { provideClientHydration, withEventReplay } from '@angular/platform-browser';
import { provideHttpClient, withFetch, withInterceptors } from '@angular/common/http';
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';
import { FormsModule } from '@angular/forms';

import { authInterceptorFn } from './auth.interceptor';

export const appConfig: ApplicationConfig = {
  providers: [
    // --- HTTP 客户端 + 拦截器/Fetch 支持 ---
    provideHttpClient(
      withInterceptors([ authInterceptorFn ]),
      withFetch()
    ),

    // --- 浏览器动画 & 表单指令 ---
    importProvidersFrom(
      FormsModule
    ),
    
    // --- 全局错误监听 & Zone 改变检测 配置 ---
    provideBrowserGlobalErrorListeners(),
    provideZoneChangeDetection({ eventCoalescing: true }),

    // --- 路由 & 客户端 hydration ---
    provideRouter(routes),
    provideClientHydration(withEventReplay())
  ]
};
