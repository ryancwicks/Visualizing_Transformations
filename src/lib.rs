use std::f64;
use vecmath::vec3_add;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use vecmath::{Vector3};
use quaternion::Quaternion;
use num::Float;

static CANVASES: [&str; 3] = ["canvas_down_x", "canvas_down_y", "canvas_down_z"];
const CANVAS_SCALE: f64 = 0.1;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(start)]
pub fn main() {
    //create Frames
    let mut frame1 = Frame::new("F_camera".into(), Some(0.25));
    let quat1 = quaternion::axis_angle([1., 0., 0.], (180.-22.)/180.*3.141592);
    //frame1.transform(&quaternion::axis_angle([0., 0., 1.], 3.14159265), &[0., 0., 0.]);
    frame1.transform(&quat1, &[0.0, 0.0, 0.0]);
    let mut frame2 = frame1.clone();
    frame2.name = "F_resultant".into();
    let quat2 = (1.23290445e-06, [-1.23290445e-06, 5.59194185e-01, 8.29036032e-01]);
    frame2.transform(&quat2, &[0.0, 0.0, -1.5336]);

    let mut frame_vec = Vec::new();
    frame_vec.push(frame1);
    frame_vec.push(frame2);

    //Draw frames.
    let mut canvas_x = ProjectedFrameDraw::new(CANVASES[0], ProjectionNormalDirection::X).unwrap();
    let mut canvas_y = ProjectedFrameDraw::new(CANVASES[1], ProjectionNormalDirection::Y).unwrap();
    let mut canvas_z = ProjectedFrameDraw::new(CANVASES[2], ProjectionNormalDirection::Z).unwrap();
    
    canvas_x.draw(&frame_vec, true);
    canvas_y.draw(&frame_vec, true);
    canvas_z.draw(&frame_vec, true);

    console_log!("Loaded");

}

/// Rectangle with a lower left and upper right corner. Used to scale the canvases to physical units
#[derive(Debug)]
struct Rectangle <T: Float> {
    x0: T,
    y0: T,
    x1: T,
    y1: T
}

impl <T> Rectangle<T> where T: Float {
    pub fn new (x0: T, y0: T, x1: T, y1: T) -> Rectangle<T> {
        let (xl, xh) = if x0 < x1 {(x0, x1)} else {(x1, x0)};
        let (yl, yh) = if y0 < y1 {(y0, y1)} else {(y1, y0)};

        return Rectangle{
            x0: xl,
            y0: yl,
            x1: xh,
            y1: yh
        }
    }
} 

#[wasm_bindgen]
#[derive(PartialEq)]
pub enum ProjectionNormalDirection {
    X,
    Y,
    Z
}

///This structure handles individual canvas drawing.
#[wasm_bindgen]
struct ProjectedFrameDraw {
    canvas: web_sys::HtmlCanvasElement,
    context: web_sys::CanvasRenderingContext2d,
    plane_normal: ProjectionNormalDirection,
    roi: Rectangle<f64>,
}

impl ProjectedFrameDraw {
    /*!
     Generate a new projected frame drawing canvas with the canvas div id canvas name, and projections onto the plane_normal plane.
     */
    pub fn new(canvas_name: &str, plane_normal: ProjectionNormalDirection) -> Result<ProjectedFrameDraw, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        
        let canvas = document.get_element_by_id(canvas_name).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        
        return Ok( 
            ProjectedFrameDraw {
                canvas: canvas,
                context: context,
                plane_normal: plane_normal,
                roi: Rectangle::new (-1., -1., 1., 1.),
            }
        )
    }

    pub fn fit_scale(&mut self, frames: &Vec<Frame>) {

        let mut total_extent = Extents::new();
        for frame in frames.iter() {
            let local_extent = frame.get_min_max();

            if total_extent.x_min > local_extent.x_min {
                total_extent.x_min = local_extent.x_min;
            }
            if total_extent.y_min > local_extent.y_min {
                total_extent.y_min = local_extent.y_min;
            }
            if total_extent.z_min > local_extent.z_min {
                total_extent.z_min = local_extent.z_min;
            }
            if total_extent.x_max < local_extent.x_max {
                total_extent.x_max = local_extent.x_max;
            }
            if total_extent.y_max < local_extent.y_max {
                total_extent.y_max = local_extent.y_max;
            }
            if total_extent.z_max < local_extent.z_max {
                total_extent.z_max = local_extent.z_max;
            }
        }

        match self.plane_normal {
            ProjectionNormalDirection::X => {
                self.roi.x0 = total_extent.y_min;
                self.roi.x1 = total_extent.y_max;
                self.roi.y0 = total_extent.z_min;
                self.roi.y1 = total_extent.z_max;
            },
            ProjectionNormalDirection::Y => {
                self.roi.x0 = total_extent.z_min;
                self.roi.x1 = total_extent.z_max;
                self.roi.y0 = total_extent.x_min;
                self.roi.y1 = total_extent.x_max;
            },
            ProjectionNormalDirection::Z => {
                self.roi.x0 = total_extent.x_min;
                self.roi.x1 = total_extent.x_max;
                self.roi.y0 = total_extent.y_min;
                self.roi.y1 = total_extent.y_max;
            }
        };

        
        self.roi.x0 = self.roi.x0 - CANVAS_SCALE*self.roi.x0;
        self.roi.x1 = self.roi.x1 + CANVAS_SCALE*self.roi.x1;
        self.roi.y0 = self.roi.y0 - CANVAS_SCALE*self.roi.y0 ;
        self.roi.y1 = self.roi.y1 + CANVAS_SCALE*self.roi.y1 ;

        let x_diff = self.roi.x1 - self.roi.x0; 
        let x_scale = x_diff / self.canvas.width() as f64;
        let y_diff = self.roi.y1 - self.roi.y0; 
        let y_scale = y_diff / self.canvas.height() as f64;

        if x_scale < y_scale {
            let xm = self.roi.x0 + x_diff/2.;
            self.roi.x0 = xm - x_diff * (y_scale/x_scale)/2.;
            self.roi.x1 = xm + (y_scale/x_scale)*x_diff/2.;
        } else {
            let ym = self.roi.y0 + y_diff/2.;
            self.roi.y0 = ym - (x_scale/y_scale)*y_diff/2.;
            self.roi.y1 = ym + (x_scale/y_scale)*y_diff/2.;
        }

        //if self.plane_normal == ProjectionNormalDirection::Y {
        //    self.roi.y0 = self.canvas.width() as f64 - self.roi.y0;
        //    self.roi.y1 = self.canvas.width() as f64 - self.roi.y1;
        //}

    }

    pub fn draw(&mut self, frames: &Vec<Frame>, clear: bool) {
        if clear == true {
            self.context.clear_rect(0.0, 0.0, self.canvas.width().into(), self.canvas.height().into());
        }

        //Find the size of the view.
        self.fit_scale(frames);

        self.context.set_stroke_style(&"black".into());
        self.context.begin_path();
        self.context.move_to (0., 0.);
        self.context.line_to (self.canvas.width().into(), 0.);
        self.context.line_to (self.canvas.width().into(), self.canvas.height().into());
        self.context.line_to (0., self.canvas.height().into());
        self.context.line_to (0., 0.);
        self.context.stroke();

        for frame in frames.iter() {
            let (xo, yo) = match self.plane_normal {
                ProjectionNormalDirection::X => self.scale(frame.origin[1], frame.origin[2]),
                ProjectionNormalDirection::Y => self.scale(frame.origin[2], frame.origin[0]),
                ProjectionNormalDirection::Z => self.scale(frame.origin[0], frame.origin[1])
            };
            let (x1, y1) = match self.plane_normal {
                ProjectionNormalDirection::X => self.scale(frame.x[1], frame.x[2]),
                ProjectionNormalDirection::Y => self.scale(frame.x[2], frame.x[0]),
                ProjectionNormalDirection::Z => self.scale(frame.x[0], frame.x[1])
            };
            let (x2, y2) = match self.plane_normal {
                ProjectionNormalDirection::X => self.scale(frame.y[1], frame.y[2]),
                ProjectionNormalDirection::Y => self.scale(frame.y[2], frame.y[0]),
                ProjectionNormalDirection::Z => self.scale(frame.y[0], frame.y[1])
            };
            let (x3, y3) = match self.plane_normal {
                ProjectionNormalDirection::X => self.scale(frame.z[1], frame.z[2]),
                ProjectionNormalDirection::Y => self.scale(frame.z[2], frame.z[0]),
                ProjectionNormalDirection::Z => self.scale(frame.z[0], frame.z[1])
            };
            
            self.context.set_stroke_style(&"red".into());
            self.context.begin_path();
            self.context.move_to (xo, yo);
            self.context.line_to (x1, y1);
            self.context.stroke();
            
            self.context.set_stroke_style(&"green".into());
            self.context.begin_path();
            self.context.move_to (xo, yo);
            self.context.line_to (x2, y2);
            self.context.stroke();
            
            self.context.set_stroke_style(&"blue".into());
            self.context.begin_path();
            self.context.move_to (xo, yo);
            self.context.line_to (x3, y3);
            self.context.stroke();

            self.context.set_stroke_style(&"black".into());
            self.context.begin_path();
            self.context.move_to (xo, yo);
            self.context.stroke_text(&frame.name, xo, yo).unwrap();
            self.context.stroke();
            
        }

    }

    pub fn scale(&self, x: f64, y: f64) -> (f64, f64) {
        let c_x0: f64 = self.canvas.width() as f64 * CANVAS_SCALE;
        let c_y0: f64 = self.canvas.height() as f64 * CANVAS_SCALE;
        let c_x1: f64 = self.canvas.width() as f64 * (1. - CANVAS_SCALE);
        let c_y1: f64 = self.canvas.height() as f64 * (1. - CANVAS_SCALE);

        let mapped_x = (x - self.roi.x0) * (c_x1 - c_x0) / (self.roi.x1 - self.roi.x0) + c_x0;
        let mapped_y = self.canvas.height() as f64 - ((y - self.roi.y0) * (c_y1 - c_y0) / (self.roi.y1 - self.roi.y0) + c_y0);
        
        return (mapped_x, mapped_y);
    }
}

#[derive(Debug)]
struct Extents {
    x_min: f64,
    y_min: f64,
    z_min: f64,
    x_max: f64,
    y_max: f64,
    z_max: f64,
}

impl Extents {
    pub fn new()->Extents {
        return Extents { 
            x_min: -0.01, 
            y_min: -0.01, 
            z_min: -0.01, 
            x_max: 0.01, 
            y_max: 0.01, 
            z_max: 0.01 
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
struct Frame {
    origin: Vector3<f64>,
    x: Vector3<f64>,
    y: Vector3<f64>,
    z: Vector3<f64>,

    name: String,
}

impl Frame {
    pub fn new (name: String, length: Option<f64>) -> Frame {
        let l = match length {
            Some(val) => val,
            None => 1.0
        };
        return Frame { 
            origin: [0.0, 0.0, 0.0], 
            x: [l, 0., 0.], 
            y: [0., l, 0.], 
            z: [0., 0., l],
            name: name,
        }
    }   

    pub fn transform (&mut self, rotation: &Quaternion<f64>, translation: &Vector3<f64>) {
        self.origin = quaternion::rotate_vector(*rotation, self.origin);
        self.origin = vec3_add(*translation, self.origin);
        self.x = quaternion::rotate_vector(*rotation, self.x);
        self.x = vec3_add(*translation, self.x);
        self.y = quaternion::rotate_vector(*rotation, self.y);
        self.y = vec3_add(*translation, self.y);
        self.z = quaternion::rotate_vector(*rotation, self.z);
        self.z = vec3_add(*translation, self.z);
    }

    pub fn get_min_max(&self) -> Extents {
        let mut mins = vec![10000., 10000., 10000.];
        let mut maxs = vec![-10000., -10000., -10000.];

        for i in 0..3 {
            if self.origin[i] < mins[i] {
                mins[i] = self.origin[i];
            }
            if self.origin[i] > maxs[i] {
                maxs[i] = self.origin[i];
            }
            if self.x[i] < mins[i] {
                mins[i] = self.x[i];
            }
            if self.x[i] > maxs[i] {
                maxs[i] = self.x[i];
            }
            if self.y[i] < mins[i] {
                mins[i] = self.y[i];
            }
            if self.y[i] > maxs[i] {
                maxs[i] = self.y[i];
            }
            if self.z[i] < mins[i] {
                mins[i] = self.z[i];
            }
            if self.z[i] > maxs[i] {
                maxs[i] = self.z[i];
            }
        }

        return Extents { 
            x_min: mins[0], 
            y_min: mins[1], 
            z_min: mins[2], 
            x_max: maxs[0], 
            y_max: maxs[1], 
            z_max: maxs[2] 
        };
    }
} 