import { Component, OnInit } from '@angular/core';
import { PortMapService, PortMapConfig, PortMapRule } from '../portmap';
import { finalize } from 'rxjs/operators';
import { CommonModule } from '@angular/common'; // Provides *ngIf, *ngFor, uppercase pipe
import { FormsModule } from '@angular/forms'; // Provides [(ngModel)] and ngForm

// --- Import Angular Material Modules ---
import { MatCardModule } from '@angular/material/card';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { MatButtonModule } from '@angular/material/button';
import { MatTableModule } from '@angular/material/table';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatIconModule } from '@angular/material/icon';


@Component({
  selector: 'app-portmap-component',
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatFormFieldModule,
    MatInputModule,
    MatSelectModule,
    MatButtonModule,
    MatTableModule,
    MatProgressSpinnerModule,
    MatIconModule
  ],
  templateUrl: './portmap-component.html',
  styleUrl: './portmap-component.css'
})

export class PortMapManagerComponent implements OnInit {

  // Component state
  config: PortMapConfig | null = null;
  isLoading = true;
  error: string | null = null;

  // For "Add New Rule" form two-way data binding
  newRule: PortMapRule = {
    protocol: 'tcp',
    external_port: 8080,
    internal_ip: '192.168.1.100',
    internal_port: 80
  };

  // For "Set Interface" form two-way data binding
  currentInterface = '';

  // Columns to display in the Material table
  displayedColumns: string[] = ['protocol', 'external_port', 'internal_ip', 'internal_port', 'actions'];

  constructor(private portMapService: PortMapService) { }

  ngOnInit(): void {
    this.loadConfig();
  }

  loadConfig(): void {
    this.isLoading = true;
    this.error = null;
    this.portMapService.getConfig().pipe(
      finalize(() => this.isLoading = false)
    ).subscribe({
      next: (data) => {
        this.config = data;
        this.currentInterface = data.external_interface;
      },
      error: (err) => {
        this.error = `Failed to load configuration: ${err.message}`;
        console.error(err);
      }
    });
  }

  onAddRule(): void {
    this.isLoading = true;
    this.error = null;
    this.portMapService.addRule(this.newRule).pipe(
      finalize(() => this.isLoading = false)
    ).subscribe({
      next: () => {
        this.loadConfig();
        this.resetNewRuleForm();
      },
      error: (err) => {
        this.error = `Failed to add rule: ${err.error?.error || err.message}`;
        console.error(err);
      }
    });
  }

  onDeleteRule(ruleToDelete: PortMapRule): void {
    if (!confirm(`Are you sure you want to delete the rule ${ruleToDelete.protocol.toUpperCase()}:${ruleToDelete.external_port}?`)) {
      return;
    }

    this.isLoading = true;
    this.error = null;
    this.portMapService.deleteRule(ruleToDelete).pipe(
      finalize(() => this.isLoading = false)
    ).subscribe({
      next: () => {
        this.loadConfig();
      },
      error: (err) => {
        this.error = `Failed to delete rule: ${err.error?.error || err.message}`;
        console.error(err);
      }
    });
  }

  onSetInterface(): void {
    this.isLoading = true;
    this.error = null;
    this.portMapService.setInterface(this.currentInterface).pipe(
      finalize(() => this.isLoading = false)
    ).subscribe({
      next: () => {
        this.loadConfig();
        alert('Network interface updated successfully!');
      },
      error: (err) => {
        this.error = `Failed to update interface: ${err.error?.error || err.message}`;
        console.error(err);
      }
    });
  }
  
  private resetNewRuleForm(): void {
    this.newRule = {
      protocol: 'tcp',
      external_port: 8080,
      internal_ip: '192.168.1.100',
      internal_port: 80
    };
  }
}
