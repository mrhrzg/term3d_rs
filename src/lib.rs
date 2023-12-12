use std::fs::File;
use std::io::{BufReader, Write};
use std::ops::Sub;
//use tui;
use ansi_term::Colour::RGB;
use obj::{load_obj, Obj, Vertex};


pub struct TriangleGeometry {
    pub tris: Vec<Triangle>
}

impl Default for TriangleGeometry {
    fn default() -> Self {
        Self {
            tris: load_example_model()
        }
    }
}

#[derive(Debug)]
pub struct Display {
    pub xdim: usize,
    pub ydim: usize,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            xdim: 180,
            ydim: 70,
        }
    }
}

pub struct Camera {
    pub shift_x: f32,
    pub shift_y: f32,
    pub zoom: f32,
    pub aspectratio: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            shift_x: -39.0,
            shift_y: -80.0,
            zoom: 1.8,
            aspectratio: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Depthpixel {
    z: f32,
    value: [f32; 3],
}

impl Default for Depthpixel {
    fn default() -> Self {
        Depthpixel {
            z: f32::NEG_INFINITY,
            value: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct V3d {
    x: f32,
    y: f32,
    z: f32,
}

impl V3d {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        V3d{x,y,z}
    }

    // x points up in the weird Obj definition
    pub fn rotate_around_x(self, theta: f32) -> Self {
        let sin_t = theta.sin();
        let cos_t = theta.cos();
        Self::new(self.x, self.y * cos_t - self.z * sin_t, self.y * sin_t + self.z * cos_t)
    }

    #[allow(dead_code)]
    pub fn abs(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) +self.z.powi(2)).sqrt()
    }

}

impl Sub for V3d {
    type Output = V3d;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x-rhs.x, self.y-rhs.y, self.z-rhs.z)
    }
}

#[derive(Debug)]
pub struct Triangle {
    pub vertices: Vec<V3d>,
    pub normals: Vec<V3d>,
}

impl Triangle {
    pub fn new(vertices: Vec<V3d>, normals: Vec<V3d>) -> Self {
        Self {vertices, normals}
    }

    pub fn rotate_x(&self, theta: f32) -> Self {
        Self::new(
            self.vertices.iter().map(|p| p.rotate_around_x(theta)).collect(),
            self.normals.iter().map(|p| p.rotate_around_x(theta)).collect()
        )
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Color(u8, u8, u8);

impl Color {
    pub fn to_web_colors(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.0, self.1, self.2)
    }
}

                                  // should be read out at the time of calculation based on the output

fn clockwise(p: &V3d, q: &V3d, r: &V3d) -> bool {
    (q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x) < 0.0
}

fn pixel_in_triangle(tri: &Triangle, pixel: (f32, f32)) -> bool {
    let a = &V3d::new(pixel.0, pixel.1, 0.0);
    if let [p, q, r] = &tri.vertices[..] {
        let orientation = clockwise(p, q, r);

        clockwise(p, q, a) == orientation
            && clockwise(q, r, a) == orientation
            && clockwise(r, p, a) == orientation
    } else {
        panic!("not a complete set of three vertices for a triangle. should not occur.");
    }
}

fn barymetric(tri: &Triangle, pixel: (f32, f32)) -> Vec<f32> {
    let (x, y) = pixel;
    let x1 = tri.vertices[0].x;
    let y1 = tri.vertices[0].y;
    let x2 = tri.vertices[1].x;
    let y2 = tri.vertices[1].y;
    let x3 = tri.vertices[2].x;
    let y3 = tri.vertices[2].y;
    let lambda1 = ((y2 - y3) * (x - x3) + (x3 - x2) * (y - y3))
        / ((y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3));
    let lambda2 = ((y3 - y1) * (x - x3) + (x1 - x3) * (y - y3))
        / ((y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3));
    let lambda3 = 1.0 - lambda1 - lambda2;
    vec![lambda1, lambda2, lambda3]
}

fn tri_interpolate(tri: &Triangle, pixel: (f32, f32)) -> Option<Depthpixel> {
    if !pixel_in_triangle(tri, pixel) {
        Some(Depthpixel {
            z: -9999.0,
            value: [1.0_f32, 1.0_f32, 1.0_f32],
        }) // white background
           //None
    } else {
        let lambdas = barymetric(tri, pixel);
        let distance = lambdas
            .iter()
            .zip(tri.vertices.iter())
            .map(|(l, v)| l * v.z)
            .sum::<f32>()
            / 3.0;
        let nx = lambdas
            .iter()
            .zip(tri.normals.iter())
            .map(|(l, n)| l * n.x)
            .sum::<f32>()
            / 3_f32;
        let ny = lambdas
            .iter()
            .zip(tri.normals.iter())
            .map(|(l, n)| l * n.y)
            .sum::<f32>()
            / 3_f32;
        let nz = lambdas
            .iter()
            .zip(tri.normals.iter())
            .map(|(l, n)| l * n.z)
            .sum::<f32>()
            / 3_f32;
        /*
        print!("Lambdas {:?}", lambdas);
        print!(" Normals {:?}", tri.normals);
        print!(" z:{:?}", distance);
        print!(" nx:{:?}", nx);
        print!(" ny:{:?}", ny);
        println!(" nz:{:?}", nz);
        */
        Some(Depthpixel {
            z: distance,
            value: [-nx, -ny, -nz],
        })
    }
}

pub fn write_to_ppm(display: &Display, zbuffer: &Vec<Vec<Depthpixel>>) {
    // https://en.m.wikipedia.org/wiki/Netpbm
    let mut file = File::create("sample_output.ppm").unwrap();
    writeln!(&mut file, "P3").unwrap();
    writeln!(
        &mut file,
        "{}",
        format_args!("{} {}", display.xdim, display.ydim)
    )
    .unwrap();
    writeln!(&mut file, "255 #max value for each color").unwrap();
    for line in zbuffer {
        let pline = line
            .iter()
            .map(|Depthpixel { z: _, value }| {
                format!(
                    "{} {} {}   ",
                    (value[0] * 256.0) as u8,
                    (value[1] * 256.0) as u8,
                    (value[2] * 256.0) as u8,
                )
            })
            .collect::<String>();
        writeln!(&mut file, "{}", pline).unwrap();
    }
}

pub fn print_to_screen(zbuffer: & [Vec<Depthpixel>]) {
    let darken = 0.4; // changes the brightness of the faux-colors
    for zbuffer_line in zbuffer.iter() {
        for zbuffer_pixel in zbuffer_line.iter() {
            print!(
                "{}",
                RGB(
                    ((1.0 - zbuffer_pixel.value[0]) * 256.0 * darken) as u8,
                    ((1.0 - zbuffer_pixel.value[1]) * 256.0 * darken) as u8,
                    ((1.0 - zbuffer_pixel.value[2]) * 256.0 * darken) as u8,
                )
                .paint("█")
            );
        }
        println!();
    }
}

pub fn flatten_buffer_to_color_frame(zbuffer: Vec<Vec<Depthpixel>>) -> Vec<Vec<String>> {
    let darken = 0.4; // changes the brightness of the faux-colors
    zbuffer.iter().map(|zbufferline| {
        zbufferline.iter().map(|zbuffer_pixel| {
            Color(((1.0 - zbuffer_pixel.value[0]) * 256.0 * darken) as u8,
                  ((1.0 - zbuffer_pixel.value[1]) * 256.0 * darken) as u8,
                  ((1.0 - zbuffer_pixel.value[2]) * 256.0 * darken) as u8,
            ).to_web_colors()
        }
        ).collect::<Vec<String>>()
    }
    ).collect()
}


pub fn render_frame(tris: &Vec<Triangle>, angle: f32, display: &Display, camera: &Camera) -> Vec<Vec<Depthpixel>> {
    let mut zbuffer = vec![vec![Depthpixel::default(); display.xdim]; display.ydim];

    for tri in tris {
        let tri = tri.rotate_x(angle);
        for (x_pix, zbuffer_line) in zbuffer.iter_mut().enumerate() {
            for (y_pix, zbuffer_pixel) in zbuffer_line.iter_mut().enumerate() {
                let x = (x_pix as f32 + camera.shift_x) * camera.zoom * camera.aspectratio;
                let y = (y_pix as f32 + camera.shift_y) * camera.zoom;

                if let Some(z_and_value) = tri_interpolate(&tri, (x, y)) {
                    if zbuffer_pixel.z < z_and_value.z {
                        *zbuffer_pixel = z_and_value.clone();
                    }
                }
            }
        }
    }
    zbuffer
}

pub fn load_example_model() -> Vec<Triangle> {
    let file = "term3d_sample_obj_5.obj".to_string();

    let input = BufReader::new(File::open(file).unwrap());
    let obj: Obj<Vertex, u64> = load_obj(input).unwrap();
    // println!( "Vertices, {:?} items: {:?}", obj.vertices.len(), obj.vertices);

    let mut tris = Vec::new();
    for ijk in obj.indices.chunks(3) {
    let t = Triangle {
    vertices: ijk
    .iter()
    .map(|i| obj.vertices[*i as usize].position)
    .map(|v| V3d::new(v[0], v[1], v[2]))
    .collect::<Vec<V3d>>(),
    normals: ijk
    .iter()
    .map(|i| obj.vertices[*i as usize].normal)
    .map(|v| V3d::new(v[0], v[1], v[2]))
    .collect::<Vec<V3d>>(),
    };
    tris.push(t);
    }
    tris
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    #[test]
    fn test_pixel_in_triangle() {
        assert!(pixel_in_triangle(
            &Triangle {
                vertices: vec![V3d::new(1.0, 0.0, 0.0), V3d::new(0.0, 1.0, 0.0), V3d::new(0.0, 0.0, 1.0)],
                normals: vec![V3d::new(0.0, 0.0, 0.0); 3],
            },
            (0.25, 0.25),
        ));
    }

    #[test]
    fn test_clockwise() {
        assert!(clockwise(
            &V3d::new(2.0, 3.0, 0.0),
            &V3d::new(6.0, 7.0, 0.0),
            &&V3d::new(4.0, -2.0, 0.0),
        ));

        assert!(!clockwise(
            &V3d::new(6.0, 7.0, 0.0),
            &V3d::new(2.0, 3.0, 0.0),
            &V3d::new(4.0, -2.0, 0.0),
        ));
    }

    #[test]
    fn test_rotation() {
        let p1 = V3d::new(7.0, 1.0, 0.0);
        let p1rot = p1.rotate_around_x(1.570796);
        let expected = V3d::new(7.0, 0.0, 1.0);
        assert_approx_eq!(p1rot, expected);

        let p1 = V3d::new(7.0, -1.0, 0.0);
        let p1rot = p1.rotate_around_x(1.570796);
        let expected = V3d::new(7.0, 0.0, -1.0);
        assert_approx_eq!(p1rot, expected);

        let p1 = V3d::new(7.0, 1.0, -1.0);
        let p1rot = p1.rotate_around_x(1.570796);
        let expected = V3d::new(7.0, 1.0, 1.0);
        assert_approx_eq!(p1rot, expected);
    }

    #[test]
    fn test_color_to_web_hex_format() {
        let c = Color { 0: 0, 1: 17, 2: 255 };
        assert_eq!(c.to_web_colors(), "#0011FF");
    }
}
