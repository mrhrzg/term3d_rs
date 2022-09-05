# term3d_rs
View 3D models in the terminal.

```bash
> cargo run [OBJ-file-name]

```

![3D models will be rendered with only gemetry taken into account](terminal.jpg "Screenshot of output of term3d in a terminal (left) and the 3D editor blender (right)")

The idea is to have a representation of the gemometry of a 3D model in the terminal. The resolution is limited, but might be sub-character with the use of baile characters. 24-bit color characters are necessary to represent the orientation of the surfaces of the model.

## Status

The aspect ratio of the terminal characters are not taken into account. This gives the output the wrong aspect ratio.

Currently also writes into a PPM file. Example output:
![3D models will be rendered with only gemetry taken into account](term3d_screenshot.jpg "Screenshot of output of term3d in PPM file (above) and the 3D editor blender (below)")


## To export from blender as OBJ use these settings

Under Transform:
* Forward: -Z Forward
* Up: -X Up

Under Geometry:
* Triangulate Faces
