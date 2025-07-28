import { Routes } from '@angular/router';
import { WelcomeComponent } from './welcome-component/welcome-component';
import { UdhcpdManagerComponent } from './udhcpd-manager/udhcpd-manager';
import { PortMapManagerComponent } from './portmap-component/portmap-component';

export const routes: Routes = [
    {path: 'home', component: WelcomeComponent},
    {path: 'udhcpd', component: UdhcpdManagerComponent},
    {path: 'portmap', component: PortMapManagerComponent},
    {path: '', component: WelcomeComponent}
];
