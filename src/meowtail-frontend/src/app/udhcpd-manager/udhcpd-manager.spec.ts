import { ComponentFixture, TestBed } from '@angular/core/testing';

import { UdhcpdManager } from './udhcpd-manager';

describe('UdhcpdManager', () => {
  let component: UdhcpdManager;
  let fixture: ComponentFixture<UdhcpdManager>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [UdhcpdManager]
    })
    .compileComponents();

    fixture = TestBed.createComponent(UdhcpdManager);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
