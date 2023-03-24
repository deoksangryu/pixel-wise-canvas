import { AfterViewInit, Component, ElementRef, OnInit, ViewChild } from '@angular/core';
import * as Module from '../wasm-package/pkg';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss']
})
export class AppComponent implements OnInit, AfterViewInit {  
  public canvas_id = '';  
  public wasm_promise!:Promise<any>;  
  private wasm_module!:any;
  private canvasWidth:number = 0;
  private canvasHeight:number = 0;

  ngOnInit() : void {
    this.wasm_promise = import('../wasm-package/pkg');
    this.wasm_promise.then((result) => {
      this.wasm_module = result;
      console.log(result.create_canvas_id());      
    });
  }

  private ratio = 0.1;    

  @ViewChild('image_canvas') private canvasWrapper: ElementRef | undefined;
  onClick() {
    // console.log("dddd");
    // this.wasm_promise = import('../wasm-package/pkg');
    // this.wasm_promise.then((result) => {
    //   this.wasm_module = result;
    //   console.log(result);      
    //   let id_string = result.create_canvas_id();
    //   console.log(id_string);
    //   result.set_canvas_id(id_string);      
    // });
    
    this.canvas_id = Module.create_canvas_id();        
    console.log(Module.create_canvas_id());
    Module.set_canvas_id("import success");
    
    Module.first_draw("../assets/002.png", this.canvas_id, this.canvasWidth, this.canvasHeight).then((result) => {      
      // let dd = result as Map<number, number>;


      let data = result as IImageInfo;
      console.log(data.width);            
      
      // let dr = Array.from(result) as number[];
      // console.log(dr);
      this.width = data.width;
      this.height = data.height;
      this.scale = data.scale;
      result.free();



      // this.width = dr[0];
      // let height = dd.get(this.width);
      // if (height) {
      //   this.height = height;
      // }
    }); 

  }

  private width = 0;
  private height = 0;

  private scale = 1.0;
  private originx = 0;
  private originy = 0;
  private isMouseDown:boolean = false;

  onMouseDown(event:any) {
    this.isMouseDown = true;
  }

  onMouseUp(event:any) {
    this.isMouseDown = false;
  }

  onMouseMove(event:any) {
    if(this.isMouseDown){
      console.log(event);
    }    
  }

  onMouseWheel(event:any) {
    console.log(event);    
    const wheelEvent = event as WheelEvent;
    wheelEvent.preventDefault();

    if(wheelEvent) {
      if(wheelEvent.deltaY > 0) {
        this.scale /= 0.8;
      }
      else{
        this.scale *= 0.8;
      }
      console.log(this.scale);
      Module.redraw(this.canvas_id, this.canvasWidth, this.canvasHeight, this.width, this.height, this.scale, 0.0, 0.0);      
    }    

    // console.log(event);
    // let canvas = event.target as HTMLCanvasElement;
    // let mousex = event.clientX - canvas.offsetLeft;
    // let mousey = event.clientY - canvas.offsetTop;
    // let wheel = event.wheelDelta/120;//n or -n

    // let zoom = 1 + wheel/2;
    // let context = canvas.getContext('2d');

    // if(context) {
    //   context.translate(
    //     this.originx,
    //     this.originy
    //   );
    //   context.scale(zoom,zoom);
    //   context.translate(
    //     -( mousex / this.scale + this.originx - mousex / ( this.scale * zoom ) ),
    //     -( mousey / this.scale + this.originy - mousey / ( this.scale * zoom ) )
    //   );

    //   this.originx = ( mousex / this.scale + this.originx - mousex / ( this.scale * zoom ) );
    //   this.originy = ( mousey / this.scale + this.originy - mousey / ( this.scale * zoom ) );
    //   this.scale *= zoom;          
    // }
  }

  ngAfterViewInit(): void {    
    if(this.canvasWrapper?.nativeElement){      
      this.canvasWidth = this.canvasWrapper.nativeElement.clientWidth;
      this.canvasHeight = this.canvasWrapper.nativeElement.clientHeight;
      console.log('set', this.canvasHeight);
    }
  }

}

export class Point2D {
  x!: number;
  y!: number;

  constructor(x: number, y: number) {
      this.x = x;
      this.y = y;
  }
}

export interface IImageInfo {
  width: number;
  height: number;
  scale: number;
}
