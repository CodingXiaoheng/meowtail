// src/app/services/port-map.service.ts

import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Observable } from 'rxjs';

// --- 数据结构定义 ---

/**
 * 端口映射规则的数据结构
 * 对应 Rust 后端的 RulePayload
 */
export interface PortMapRule {
  protocol: string;
  external_port: number;
  internal_ip: string;
  internal_port: number;
}

/**
 * 完整配置的数据结构
 * 对应 GET /config 的返回类型
 */
export interface PortMapConfig {
  external_interface: string;
  rules: PortMapRule[];
}


// --- Angular Service 实现 ---

@Injectable({
  providedIn: 'root',
})
export class PortMapService {
  // 后端 API 的基本路径
  private readonly apiUrl = '/api/portmap'; // 假设 Angular 通过代理访问后端

  constructor(private http: HttpClient) {}

  /**
   * GET /portmap/config
   * 获取当前端口映射的完整配置（包括网络接口和所有规则）
   * @returns {Observable<PortMapConfig>}
   */
  getConfig(): Observable<PortMapConfig> {
    return this.http.get<PortMapConfig>(`${this.apiUrl}/config`);
  }

  /**
   * POST /portmap/rule
   * 添加一条新的端口映射规则
   * @param rule {PortMapRule} - 要添加的规则对象
   * @returns {Observable<any>}
   */
  addRule(rule: PortMapRule): Observable<any> {
    return this.http.post<any>(`${this.apiUrl}/rule`, rule);
  }

  /**
   * DELETE /portmap/rule
   * 删除一条现有的端口映射规则
   * @param rule {PortMapRule} - 要删除的规则对象
   * @returns {Observable<any>}
   */
  deleteRule(rule: PortMapRule): Observable<any> {
    // HTTP DELETE 请求通常不携带 body，但 actix-web 和 Angular HttpClient 都支持
    const httpOptions = {
      headers: new HttpHeaders({ 'Content-Type': 'application/json' }),
      body: rule,
    };
    return this.http.delete<any>(`${this.apiUrl}/rule`, httpOptions);
  }

  /**
   * POST /portmap/interface
   * 设置用于端口映射的网络接口
   * @param interfaceName {string} - 网络接口的名称
   * @returns {Observable<any>}
   */
  setInterface(interfaceName: string): Observable<any> {
    const payload = { interface: interfaceName };
    return this.http.post<any>(`${this.apiUrl}/interface`, payload);
  }
}