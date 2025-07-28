import { TestBed } from '@angular/core/testing';

import { UdhcpdManager } from './udhcpd-manager';

describe('UdhcpdManager', () => {
  let service: UdhcpdManager;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(UdhcpdManager);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
