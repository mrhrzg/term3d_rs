use term3drender::*;

use obj::{load_obj, Obj, Vertex};
use std::env;
use std::fs::File;
use std::io::BufReader;
//use tui;
use ansi_term::Colour::RGB;
use std::time::Instant;


static FONTASPECTRATIO: f32 = 1.9; // terminal characters are not a wide as they are high. Ideally, this


fn main() {

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

    let enhance = 1.0;
    let camera_zoom = 1.8 / enhance;
    let camerashift_x = -39.0 * enhance;
    let camerashift_y = -80.0 * enhance;

    let aspectratio = if to_file { 1.0 } else { FONTASPECTRATIO };
    let camera = Camera {
        shift_x: camerashift_x,
        shift_y: camerashift_y,
        zoom: camera_zoom,
        aspectratio,
    };

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

    let display = Display {
        xdim: 180 * enhance as usize,
        ydim: 70 * enhance as usize,
    };
    println!("Number of triangles: {}", &tris.len());
    // position camera. Currently the camera is fixed to the x-y plane. One can move it in
    // the x-y plane and map the pixels to a larger or smaller region of world-space.
    // println!("{:?}", tris[0]);



    let mut now;
    for frame in 1..=60 {
        now = Instant::now();
        let angle = 0.1 * frame as f32;
        let zbuffer = render_frame(&tris, angle, &display, &camera);


        if to_file {
            // write the data to file
            write_to_ppm(&display, &zbuffer.clone());
        } else {
            // display the data
            print!("{}[2J", 27 as char); // clear screen
            print_to_screen(&zbuffer);

        }
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
        println!("time per triangle: {:.2?}", elapsed / tris.len() as u32);
    }
}
