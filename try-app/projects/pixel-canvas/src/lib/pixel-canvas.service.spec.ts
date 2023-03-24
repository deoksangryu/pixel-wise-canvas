import { TestBed } from '@angular/core/testing';

import { PixelCanvasService } from './pixel-canvas.service';

describe('PixelCanvasService', () => {
  let service: PixelCanvasService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(PixelCanvasService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
