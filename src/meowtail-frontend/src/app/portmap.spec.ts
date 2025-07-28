import { TestBed } from '@angular/core/testing';

import { Portmap } from './portmap';

describe('Portmap', () => {
  let service: Portmap;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(Portmap);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
