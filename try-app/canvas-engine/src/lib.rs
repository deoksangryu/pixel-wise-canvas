use wasm_bindgen::{prelude::*, JsCast, Clamped, JsValue};
use uuid::Uuid;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, HtmlImageElement, ImageData, Response, WheelEvent};
use image::GenericImageView;
use wasm_bindgen_futures::JsFuture;
use futures_channel::oneshot;
use js_sys::{Promise, Uint8ClampedArray, WebAssembly, Function, Uint8Array};
use rayon::prelude::*;
use rand::Rng;    
use itertools::Itertools;
use itertools::concat;
use gloo_utils::format::JsValueSerdeExt;
use std::collections::HashMap;
use serde::ser::{Serialize, Serializer, SerializeStruct};

#[wasm_bindgen]
pub async fn fetch_url_binary(url: String) -> Result<Uint8Array, JsValue> {
    let window = web_sys::window().unwrap(); // Browser window
    let promise = JsFuture::from(window.fetch_with_str(&url)); // File fetch promise
    let result = promise.await?; // Await fulfillment of fetch
    let response: web_sys::Response = result.dyn_into().unwrap(); // Type casting
    let image_data = JsFuture::from(response.array_buffer()?).await?; // Get text
    Ok(Uint8Array::new(&image_data))
}


// fn on_mouse_wheel(event: &WheelEvent, canvas: &HtmlCanvasElement) {
//     let ctx = canvas
//         .get_context("2d")
//         .unwrap()
//         .unwrap()
//         .dyn_into::<CanvasRenderingContext2d>()
//         .unwrap();

//     // Get the current transformation matrix of the canvas
//     let current_transform = ctx.get_transform();

//     // Calculate the scale factor based on the direction of the wheel
//     let scale_factor = if event.delta_y() < 0.0 { 1.1 } else { 0.9 };

//     // Apply the scale factor to the current transformation matrix
//     let new_transform = current_transform.scale(scale_factor, scale_factor);

//     // Set the new transformation matrix on the canvas
//     ctx.set_transform(&new_transform);
// }

fn zoom(canvas: &HtmlCanvasElement, scale: f64) -> Result<(), JsValue> {
    let ctx = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;
    
    ctx.scale(scale, scale)?;
    
    Ok(())
}

#[wasm_bindgen]
pub fn redraw(canvas: String, canvas_width: f64, canvas_height: f64, width: f64, height: f64, scale: f64, translate_x: f64, translate_y: f64) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().expect("Could not get document");
    let fake_canvas = document
        .get_element_by_id("fake_canvas_temp")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    let real_canvas = document
        .get_element_by_id(&canvas)
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = real_canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
        
    context.reset_transform();
    context.clear_rect(0.0, 0.0, canvas_width, canvas_height);    
    context.save();    
    context.scale(scale, scale);
    context.draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        &fake_canvas,
        0.0,
        0.0,
        width,
        height,
        0.0,
        0.0,
        width,
        height
    );    
    context.restore();
    Ok(())
}

#[wasm_bindgen]
pub async fn first_draw(url: String, canvas: String, canvas_width: f64, canvas_height: f64) -> Result<JsValue, JsValue> {
    let binary = fetch_url_binary(url).await?;
    let altbuf = binary.to_vec();

    // Convert the png encoded bytes to an rgba pixel buffer (given the PNG is actually in 8byte RGBA format).
    let image = image::load_from_memory_with_format(&altbuf, image::ImageFormat::Png).unwrap();
    let mut rgba_image = image.to_rgba8();

    // I suppose this is what you tried to do in your original loop
    // judging by the function name:
    for (_, _, pixel) in rgba_image.enumerate_pixels_mut() {
        if pixel[0] > 0 {
            *pixel = image::Rgba([pixel[0], pixel[1], pixel[2], pixel[3]]);
        }
    }

    let window = web_sys::window().unwrap();
    let document = window.document().expect("Could not get document");

    // let fake_canvas = document.create_element("canvas")?.dyn_into::<HtmlCanvasElement>()?;

    let fake_canvas = document
        .get_element_by_id("fake_canvas_temp")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    fake_canvas.set_width(image.width());
    fake_canvas.set_height(image.height());
    let fake_context = fake_canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    let clamped_buf: Clamped<&[u8]> = Clamped(rgba_image.as_raw());
    let image_data_temp = 
        ImageData::new_with_u8_clamped_array_and_sh(clamped_buf, image.width(), image.height())?;    
    fake_context.put_image_data(&image_data_temp, 0.0, 0.0)?;
        
    let canvas = document
        .get_element_by_id(&canvas)
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    // let new_width = image.width() as f64 * 0.5;
    // let new_height = image.height() as f64 * 0.5;

    canvas.set_width(canvas_width as u32);
    canvas.set_height(canvas_height as u32);

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
    // let clamped_buf: Clamped<&[u8]> = Clamped(rgba_image.as_raw());
    // let image_data_temp = 
    //     ImageData::new_with_u8_clamped_array_and_sh(clamped_buf, image.width(), image.height())?;    
    // context.put_image_data(&image_data_temp, 0.0, 0.0)?;

    let mut ratio:f64 = 0.0;

    if image.width() >= image.height() {
        ratio = canvas_width / image.width() as f64;
    }
    else {
        ratio = canvas_height / image.height() as f64;
    }

    let translate_x = (canvas_width - image.width() as f64 * ratio) / 2.0;
    let translate_y = (canvas_height - image.height() as f64 * ratio) / 2.0;

    context.translate(translate_x, translate_y);
    context.scale(ratio, ratio);    
    // context.clear_rect(0.0, 0.0, image.width() as f64, image.height() as f64);    

    // let new_width = image.width() as f64 * 0.5;
    // let new_height = image.height() as f64 * 0.5;

    // let on_wheel_callback = Closure::wrap(Box::new(move |event: &WheelEvent| {
    //     log("wheel works");
    // }) as Box<dyn FnMut(_)>);

    // canvas.add_event_listener_with_callback("wheel", on_wheel_callback.as_ref().unchecked_ref())?;
    // on_wheel_callback.forget();    

    context.draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        &fake_canvas,
        0.0,
        0.0,
        image.width() as f64,
        image.height() as f64,
        0.0,
        0.0,
        image.width() as f64,
        image.height() as f64
    );
    context.save();
    context.restore();
    let mut datas:Vec<f64> = Vec::new();
    datas.push(image.width() as f64);
    datas.push(image.height() as f64);
    datas.push(ratio);
    let mut size = HashMap::new();
    size.insert(image.width(), image.height());    

    let mut info:ImageInfo = ImageInfo { width: image.width() as f64, height: image.height() as f64, scale: ratio };

    Ok(serde_wasm_bindgen::to_value(&info).unwrap())    
}


#[wasm_bindgen]
pub struct ImageInfo {
    width: f64,
    height: f64,
    scale: f64
}

impl Serialize for ImageInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("ImageInfo", 3)?;
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;
        state.serialize_field("scale", &self.scale)?;
        state.end()
    }
}

#[wasm_bindgen]
impl ImageInfo {
    fn width(&self) -> f64 {
        return self.width;
    }
}

#[wasm_bindgen]
#[derive(PartialEq, Eq, Ord, PartialOrd, Debug, Clone)]
pub struct Point {
    x:i32,
    y:i32    
}

#[wasm_bindgen]
pub fn create_canvas_id() -> String {
    return Uuid::new_v4().to_string();
}

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_size(a: usize);
}

#[wasm_bindgen]
pub fn set_canvas_id(id:&str) {
    log(id);
}

#[wasm_bindgen]
pub fn multitest() {    
    log("Start test");
    // use std::sync::mpsc::{Sender, Receiver};
    // use std::sync::mpsc;
    let mut _pt_vec:Vec<i32> = Vec::new();
    for i in 0..500 {
        _pt_vec.push(i);
    }   
    log("ready");
    // let (tx, rx): (Sender<Vec<Point>>, Receiver<Vec<Point>>) = mpsc::channel();
    let mut all_pts:Vec<Vec<Point>>= Vec::new();    

    //let now = Instant::now();  

    let mut alls:Vec<Vec<Point>> = _pt_vec.par_iter().map(|p| get_all_points_from_point(25, *p, *p)).collect();
    // let dd = _pt_vec.into_par_iter().for_each(|p| {
    //     let mut _pts:Vec<Point> = get_all_points_from_point(25, p, p).clone();                   
    //     // s.send(_pts).unwrap();
    // });
    // for i in &alls {
    //     log_size(i.len());
    // }    
    let mut _real:Vec<Point> = concat(alls);
    _real.sort();
    _real.dedup_by(|a, b| a.x == b.x && a.y == b.y);
    // log_size(_real.len());
    log("processing");
    // let mut all:Vec<Vec<Point>> = rx.iter().collect();    
    //let elapsed = now.elapsed();            
    log("end");
}

// #[wasm_bindgen]
// pub fn test() {        
//     let mut _pt_vec:Vec<i32> = Vec::new();
//     for i in 0..500 {
//         _pt_vec.push(i);
//     }   
//     let mut all_pts:Vec<Vec<Point>>= Vec::new();

//     let mut handles:Vec<JoinHandle<Vec<Point>>> = Vec::with_capacity(_pt_vec.len());        

//     for _i in _pt_vec {

//         let mut rng = rand::thread_rng();
//         let x = rng.gen_range(0..512);
//         let y = rng.gen_range(0..512);
//         let r:i32 = 25;
                                     
//         handles.push(thread::spawn(move || {                              
//             let _pts = get_all_points_from_point(r, _i, _i);             
//             return _pts;                         
//         }));                
//     }         
    
//     for handle in handles {
//         let result = handle.join().unwrap();             
//         all_pts.push(result);         
//     }    

//     let mut _real = concat(all_pts);
//     _real.sort();    
//     _real.dedup_by(|a, b| a.x == b.x && a.y == b.y);   

//     // log(&_real.len().to_string());
// }

fn get_all_points_from_point(r:i32, x_c:i32, y_c:i32) -> Vec<Point> {    
    let mut _result:Vec<Point> = Vec::new();                     

    for i in -r-1 .. r+1 {
        for j in -r-1 .. r+1 {
            if i.pow(2) + j.pow(2) <= r.pow(2) + 1 {
                _result.push(Point { x:i + x_c, y:j + y_c });
            }            
        }
    }
            
    return _result;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        
    }
}
