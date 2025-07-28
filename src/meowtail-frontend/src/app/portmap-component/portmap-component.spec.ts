import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PortmapComponent } from './portmap-component';

describe('PortmapComponent', () => {
  let component: PortmapComponent;
  let fixture: ComponentFixture<PortmapComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortmapComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PortmapComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
