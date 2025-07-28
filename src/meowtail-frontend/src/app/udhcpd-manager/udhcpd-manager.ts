import { Component, OnInit } from '@angular/core';
import { FormBuilder, FormGroup, FormArray, Validators, FormControl } from '@angular/forms';
import { CommonModule } from '@angular/common';
import { ReactiveFormsModule } from '@angular/forms';
import { MatToolbarModule } from '@angular/material/toolbar';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatCardModule } from '@angular/material/card';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatTableModule } from '@angular/material/table';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { UdhcpdManagerService } from '../udhcpd-manager';

type Lease = { mac: string; ip: string };
type RangeCfg = { start: string; end: string };
interface UdhcpdConfig {
  range: RangeCfg;
  gateway: string;
  subnet: string;
  interface: string;
  dnsServers: string[];
  staticLeases: Lease[];
}

@Component({
  selector: 'app-udhcpd-manager',
  standalone: true,
  imports: [
    CommonModule,
    ReactiveFormsModule,
    MatToolbarModule,
    MatButtonModule,
    MatIconModule,
    MatCardModule,
    MatFormFieldModule,
    MatInputModule,
    MatTableModule,
    MatSnackBarModule,
  ],
  templateUrl: './udhcpd-manager.html',
  styleUrls: ['./udhcpd-manager.css'],
})
export class UdhcpdManagerComponent implements OnInit {
  status = false;
  config?: UdhcpdConfig;

  rangeForm: FormGroup;
  gatewayForm: FormGroup;
  subnetForm: FormGroup;
  interfaceForm: FormGroup;
  dnsForm: FormGroup;
  leaseForm: FormGroup;

  leases: Lease[] = [];
  displayedColumns: string[] = ['mac', 'ip', 'actions'];

  public trackByIndex(index: number): number { return index; }

  constructor(
    private service: UdhcpdManagerService,
    private fb: FormBuilder,
    private snackBar: MatSnackBar
  ) {
    this.rangeForm = this.fb.group({
      start: this.fb.nonNullable.control('', Validators.required),
      end:   this.fb.nonNullable.control('', Validators.required),
    });

    this.gatewayForm = this.fb.group({
      gateway: this.fb.control('', { nonNullable: true, validators: [Validators.required] }),
    });

    this.subnetForm = this.fb.group({
      subnet: this.fb.control('', { nonNullable: true, validators: [Validators.required] }),
    });

    this.interfaceForm = this.fb.group({
      interface: this.fb.control('', { nonNullable: true, validators: [Validators.required] }),
    });

    this.dnsForm = this.fb.group({
      servers: this.fb.array<FormControl<string>>([
        this.fb.control('', { nonNullable: true, validators: [Validators.required] }),
      ]),
    });

    this.leaseForm = this.fb.group({
      mac: this.fb.control('', { nonNullable: true, validators: [Validators.required] }),
      ip: this.fb.control('', { nonNullable: true, validators: [Validators.required] }),
    });
  }

  get servers(): FormArray<FormControl<string>> {
    return this.dnsForm.get('servers') as FormArray<FormControl<string>>;
  }

  ngOnInit(): void {
    this.reloadStatus();
    this.loadConfig();
  }

  reloadStatus(): void {
    this.service.status().subscribe({
      next: (res) => (this.status = !!res?.running),
      error: () => (this.status = false),
    });
  }

  /**
   * ★ 从后端加载配置并回显到页面 (重构版)
   * 重构此方法以更可靠地处理来自服务器的各种数据格式（例如，DNS 服务器可能是单个字符串或数组），
   * 并使代码逻辑更清晰、更健壮。
   */
  loadConfig(): void {
    this.service.getConfig().subscribe({
      next: (raw) => {
        // 1. 将服务器可能返回的不同字段名和数据类型，统一为标准化的内部格式。
        
        // 兼容 DNS 服务器字段（dns, dnsServers, dns_servers），并确保结果始终为字符串数组
        const rawDns = raw?.dnsServers ?? raw?.dns ?? raw?.dns_servers;
        const normalizedDns = rawDns ? (Array.isArray(rawDns) ? rawDns : [String(rawDns)]) : [];

        // 兼容静态租约字段（leases, staticLeases, static_leases），并确保结果始终为数组
        const rawLeases = raw?.staticLeases ?? raw?.leases ?? raw?.static_leases;
        const normalizedLeases = Array.isArray(rawLeases) ? rawLeases : [];

        const cfg: UdhcpdConfig = {
          range: raw?.range ?? { start: raw?.start ?? '', end: raw?.end ?? '' },
          // 兼容网关字段 (gateway, gw, router)
          gateway: raw?.gateway ?? raw?.gw ?? raw?.router ?? '',
          // 兼容子网掩码字段 (subnet, netmask, subnet_mask)
          subnet: raw?.subnet ?? raw?.netmask ?? raw?.subnet_mask ?? '',
          // 兼容接口字段 (interface, ifname)
          interface: raw?.interface ?? raw?.ifname ?? '',
          dnsServers: normalizedDns.filter(dns => !!dns), // 过滤掉无效的DNS条目
          staticLeases: normalizedLeases,
        };

        this.config = cfg;

        // 2. 使用标准化的配置数据安全地更新各个表单。
        this.rangeForm.patchValue(cfg.range ?? { start: '', end: '' }, { emitEvent: false });
        this.gatewayForm.patchValue({ gateway: cfg.gateway ?? '' }, { emitEvent: false });
        this.subnetForm.patchValue({ subnet: cfg.subnet ?? '' }, { emitEvent: false });
        this.interfaceForm.patchValue({ interface: cfg.interface ?? '' }, { emitEvent: false });

        // 3. 动态重建 DNS 服务器的 FormArray。(★ 修改点)
        // 使用 map 创建所有需要的 FormControl
        const dnsControls = cfg.dnsServers.map(server => 
          this.fb.control(server, { nonNullable: true, validators: [Validators.required] })
        );

        // 如果没有 DNS 服务器，则创建一个空的输入框
        if (dnsControls.length === 0) {
          dnsControls.push(this.fb.control('', { nonNullable: true, validators: [Validators.required] }));
        }

        // 使用 setControl 一次性替换整个 FormArray，这有助于避免潜在的变更检测问题
        this.dnsForm.setControl('servers', this.fb.array(dnsControls), { emitEvent: false });

        // 4. 更新静态租约表格的数据源。
        this.leases = cfg.staticLeases;
      },
      error: (err) => {
        this.notify('加载配置失败');
        console.error('getConfig error', err);
      },
    });
  }

  start(): void {
    this.service.start().subscribe(res => this.notify(res?.status ?? 'ok', true));
  }
  stop(): void {
    this.service.stop().subscribe(res => this.notify(res?.status ?? 'ok', true));
  }
  restart(): void {
    this.service.restart().subscribe(res => this.notify(res?.status ?? 'ok', true));
  }

  updateRange(): void {
    if (!this.rangeForm.valid) return;
    const { start, end } = this.rangeForm.getRawValue();
    this.service.setRange(start, end).subscribe(res => {
      this.notify(res?.status ?? 'ok');
      this.loadConfig();
    });
  }

  updateGateway(): void {
    if (!this.gatewayForm.valid) return;
    const { gateway } = this.gatewayForm.getRawValue();
    this.service.setGateway(gateway).subscribe(res => {
      this.notify(res?.status ?? 'ok');
      this.loadConfig();
    });
  }

  updateSubnet(): void {
    if (!this.subnetForm.valid) return;
    const { subnet } = this.subnetForm.getRawValue();
    this.service.setSubnet(subnet).subscribe(res => {
      this.notify(res?.status ?? 'ok');
      this.loadConfig();
    });
  }

  updateInterface(): void {
    if (!this.interfaceForm.valid) return;
    const { interface: ifname } = this.interfaceForm.getRawValue() as any;
    this.service.setInterface(ifname).subscribe(res => {
      this.notify(res?.status ?? 'ok');
      this.loadConfig();
    });
  }

  addDnsField(): void {
    this.servers.push(this.fb.control('', { nonNullable: true, validators: [Validators.required] }));
  }

  removeDnsField(i: number): void {
    if (this.servers.length > 1) {
      this.servers.removeAt(i);
    } else {
      this.servers.at(0).setValue('');
    }
  }

  updateDns(): void {
    const servers = this.servers.value.filter(v => !!v && v.trim().length > 0);
    this.service.setDns(servers as string[]).subscribe(res => {
      this.notify(res?.status ?? 'ok');
      this.loadConfig();
    });
  }

  addLease(): void {
    if (!this.leaseForm.valid) return;
    const { mac, ip } = this.leaseForm.getRawValue();
    this.service.addLease(mac, ip).subscribe(res => {
      this.notify(res?.status ?? 'ok');
      this.leaseForm.reset();
      this.loadConfig();
    });
  }

  removeLease(mac: string): void {
    this.service.removeLease(mac).subscribe(res => {
      this.notify(res?.status ?? 'ok');
      this.loadConfig();
    });
  }

  private notify(message: string, refreshStatus = false): void {
    this.snackBar.open(message, 'Close', { duration: 2000 });
    if (refreshStatus) this.reloadStatus();
  }
}
