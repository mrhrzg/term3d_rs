use obj::{load_obj, Obj};
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
//use tui;

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
            z: f32::INFINITY,
            value: [0.0, 0.0, 0.0],
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
            z: 9999.0,
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
            .map(|(l, n)| l * n[2] * 10.0)
            .sum::<f32>()
            / 3_f32;
        Some(Depthbuffer {
            z: distance,
            value: [nx, ny, nz],
        })
    }
}

fn write_to_ppm(display: Display, zbuffer: Vec<Vec<Depthbuffer>>) {
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
                    value[0] as u8 * 255,
                    value[1] as u8 * 255,
                    value[2] as u8 * 255
                )
            })
            .collect::<String>();
        writeln!(&mut file, "{}", pline).unwrap();
    }
}

fn main() {
    let input = BufReader::new(File::open("obj_cube3.obj").unwrap());
    let obj: Obj = load_obj(input).unwrap();

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
        xdim: 200,
        ydim: 200,
    };
    // TODO: rotate world or camera
    println!("Number of triangles: {}", &tris.len());
    // position camera. Currently the camera is fixed to the x-y plane. One can move it in
    // the x-y plane and map the pixels to a larger or smaller region of worldspace.
    let camera_zoom = 1.5;
    let camerashift_x = -70.0;
    let camerashift_y = -70.0;

    println!("{:?}", tris[0]);

    let mut zbuffer = vec![vec![Depthbuffer::default(); display.xdim]; display.ydim];
    for tri in tris {
        for (x_pix, zbuffer_line) in zbuffer.iter_mut().enumerate() {
            for (y_pix, zbuffer_pixel) in zbuffer_line.iter_mut().enumerate() {
                let x = (x_pix as f32 + camerashift_x) * camera_zoom;
                let y = (y_pix as f32 + camerashift_y) * camera_zoom;

                if let Some(z_and_value) = tri_interpolate(&tri, (x, y)) {
                    if zbuffer_pixel.z > z_and_value.z {
                        *zbuffer_pixel = z_and_value
                    }
                }
            }
        }
    }
    // display the data
    write_to_ppm(display, zbuffer);
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
