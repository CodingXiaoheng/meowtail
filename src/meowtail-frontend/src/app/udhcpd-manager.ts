import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

@Injectable({
  providedIn: 'root'
})

export class UdhcpdManagerService {
  private baseUrl = '/api/udhcpd';

  constructor(private http: HttpClient) {}

  start(): Observable<any> {
    return this.http.post(`${this.baseUrl}/start`, {});
  }

  stop(): Observable<any> {
    return this.http.post(`${this.baseUrl}/stop`, {});
  }

  restart(): Observable<any> {
    return this.http.post(`${this.baseUrl}/restart`, {});
  }

  status(): Observable<{ running: boolean }> {
    return this.http.get<{ running: boolean }>(`${this.baseUrl}/status`);
  }

  getConfig(): Observable<any> {
    return this.http.get<any>(`${this.baseUrl}/config`);
  }

  setRange(start: string, end: string): Observable<any> {
    return this.http.post(`${this.baseUrl}/config/range`, { start, end });
  }

  setGateway(gateway: string): Observable<any> {
    return this.http.post(`${this.baseUrl}/config/gateway`, { gateway });
  }

  setSubnet(subnet: string): Observable<any> {
    return this.http.post(`${this.baseUrl}/config/subnet`, { subnet });
  }

  setInterface(interfaceName: string): Observable<any> {
    return this.http.post(`${this.baseUrl}/config/interface`, { interface: interfaceName });
  }

  setDns(servers: string[]): Observable<any> {
    return this.http.post(`${this.baseUrl}/config/dns`, { servers });
  }

  addLease(mac: string, ip: string): Observable<any> {
    return this.http.post(`${this.baseUrl}/config/lease`, { mac, ip });
  }

  removeLease(mac: string): Observable<any> {
    return this.http.delete(`${this.baseUrl}/config/lease`, { body: { mac } });
  }
}
