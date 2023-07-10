use obj::{load_obj, Obj, Vertex};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
//use tui;
use ansi_term::Colour::RGB;
use std::time::Instant;

#[derive(Debug)]
struct Display {
    xdim: usize,
    ydim: usize,
}

#[derive(Debug, Clone)]
struct Depthbuffer {
    z: f32,
    value: [f32; 3],
}

impl Default for Depthbuffer {
    fn default() -> Self {
        Depthbuffer {
            z: f32::NEG_INFINITY,
            value: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Debug)]
struct Triangle {
    vertices: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
}

#[derive(Debug, Default)]
struct Color(u8, u8, u8);

static FONTASPECTRATIO: f32 = 1.9; // terminal characters are not a wide as they are high. Ideally, this
                                   // should be read out at the time of calculation based on the output

fn clockwise(p: &[f32; 3], q: &[f32; 3], r: &[f32; 3]) -> bool {
    (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]) < 0.0
}

fn pixel_in_triangle(tri: &Triangle, pixel: (f32, f32)) -> bool {
    let a = &[pixel.0, pixel.1, 0.0];
    if let [p, q, r] = &tri.vertices[..] {
        let orientation = clockwise(p, q, r);

        clockwise(p, q, a) == orientation
            && clockwise(q, r, a) == orientation
            && clockwise(r, p, a) == orientation
    } else {
        panic!("not a complet set of three vertices for a triangle. should not happend.");
    }
}

fn barymetric(tri: &Triangle, pixel: (f32, f32)) -> Vec<f32> {
    let (x, y) = pixel;
    let x1 = tri.vertices[0][0];
    let y1 = tri.vertices[0][1];
    let x2 = tri.vertices[1][0];
    let y2 = tri.vertices[1][1];
    let x3 = tri.vertices[2][0];
    let y3 = tri.vertices[2][1];
    let lambda1 = ((y2 - y3) * (x - x3) + (x3 - x2) * (y - y3))
        / ((y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3));
    let lambda2 = ((y3 - y1) * (x - x3) + (x1 - x3) * (y - y3))
        / ((y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3));
    let lambda3 = 1.0 - lambda1 - lambda2;
    vec![lambda1, lambda2, lambda3]
}

fn tri_interpolate(tri: &Triangle, pixel: (f32, f32)) -> Option<Depthbuffer> {
    if !pixel_in_triangle(tri, pixel) {
        Some(Depthbuffer {
            z: -9999.0,
            value: [1.0_f32, 1.0_f32, 1.0_f32],
        }) // white background
           //None
    } else {
        let lambdas = barymetric(tri, pixel);
        let distance = lambdas
            .iter()
            .zip(tri.vertices.iter())
            .map(|(l, v)| l * v[2])
            .sum::<f32>()
            / 3.0;
        let nx = lambdas
            .iter()
            .zip(tri.normals.iter())
            .map(|(l, n)| l * n[0])
            .sum::<f32>()
            / 3_f32;
        let ny = lambdas
            .iter()
            .zip(tri.normals.iter())
            .map(|(l, n)| l * n[1])
            .sum::<f32>()
            / 3_f32;
        let nz = lambdas
            .iter()
            .zip(tri.normals.iter())
            .map(|(l, n)| l * n[2])
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
        Some(Depthbuffer {
            z: distance,
            value: [-nx, -ny, -nz],
        })
    }
}

fn write_to_ppm(display: Display, zbuffer: Vec<Vec<Depthbuffer>>) {
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
            .map(|Depthbuffer { z: _, value }| {
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

fn print_to_screen(mut zbuffer: Vec<Vec<Depthbuffer>>) {
    let darken = 0.4; // changes the brightness of the faux-colors
    for zbuffer_line in zbuffer.iter_mut() {
        for zbuffer_pixel in zbuffer_line.iter_mut() {
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

fn main() {
    let now = Instant::now();
    let args: Vec<String> = env::args().collect();
    let to_file = if args.len() >= 2 {
        args[1].clone() == "to_file"
    } else {
        false
    };
    let file = if args.len() >= 3 {
        args[2].clone()
    } else {
        "term3d_sample_obj_5.obj".to_string()
    };
    println!(
        "Previewing 3D file {}",
        RGB(70, 130, 180).paint(file.clone())
    );
    let input = BufReader::new(File::open(file).unwrap());
    let enhance = 1.0;
    let camera_zoom = 1.8 / enhance;
    let camerashift_x = -39.0 * enhance;
    let camerashift_y = -80.0 * enhance;

    let obj: Obj<Vertex, u64> = load_obj(input).unwrap();
    // println!( "Vertices, {:?} items: {:?}", obj.vertices.len(), obj.vertices);

    let mut tris = Vec::new();
    for ijk in obj.indices.chunks(3) {
        let t = Triangle {
            vertices: ijk
                .iter()
                .map(|i| obj.vertices[*i as usize].position)
                .collect::<Vec<[f32; 3]>>(),
            normals: ijk
                .iter()
                .map(|i| obj.vertices[*i as usize].normal)
                .collect::<Vec<[f32; 3]>>(),
        };
        tris.push(t);
    }

    let display = Display {
        xdim: 180 * enhance as usize,
        ydim: 70 * enhance as usize,
    };
    // TODO: rotate world or camera
    println!("Number of triangles: {}", &tris.len());
    // position camera. Currently the camera is fixed to the x-y plane. One can move it in
    // the x-y plane and map the pixels to a larger or smaller region of worldspace.
    // println!("{:?}", tris[0]);

    let aspectratio = if to_file { 1.0 } else { FONTASPECTRATIO };

    let mut zbuffer = vec![vec![Depthbuffer::default(); display.xdim]; display.ydim];
    for tri in &tris {
        for (x_pix, zbuffer_line) in zbuffer.iter_mut().enumerate() {
            for (y_pix, zbuffer_pixel) in zbuffer_line.iter_mut().enumerate() {
                let x = (x_pix as f32 + camerashift_x) * camera_zoom * aspectratio;
                let y = (y_pix as f32 + camerashift_y) * camera_zoom;

                if let Some(z_and_value) = tri_interpolate(&tri, (x, y)) {
                    if zbuffer_pixel.z < z_and_value.z {
                        *zbuffer_pixel = z_and_value.clone();
                    }
                }
            }
        }
    }

    if to_file {
        // write the data to file
        write_to_ppm(display, zbuffer.clone());
    } else {
        // display the data
        print_to_screen(zbuffer);
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    println!("time per triangle: {:.2?}", elapsed / tris.len() as u32);
}

#[test]
fn test_pixel_in_triangle() {
    assert!(pixel_in_triangle(
        &Triangle {
            vertices: vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]],
            normals: vec![[0.0; 3]; 3],
        },
        (0.25, 0.25),
    ));
}

#[test]
fn test_clockwise() {
    assert!(clockwise(
        &[2.0, 3.0, 0.0],
        &[6.0, 7.0, 0.0],
        &[4.0, -2.0, 0.0],
    ));

    assert!(!clockwise(
        &[6.0, 7.0, 0.0],
        &[2.0, 3.0, 0.0],
        &[4.0, -2.0, 0.0],
    ));
}
