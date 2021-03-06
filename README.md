
# *Soft*ware Triangle Renderer

This is a toy library, that was written in a weekend, which implements triangle renderer
that runs on the CPU instead of the GPU. 
The library exposes an API that allows the user to render a triangulated model, and
exposes control over the vertex and fragment (pixel) stages of the render
pipeline through programmable shaders.


## Demo

![Screenshot of the renderer in action](doc/screenshot.png)

As you can see we are drawing a cube with a gradient, and texturing it with a
checker texture. This is all done in the programmable shader, all while running
at a "respectable" 225 fps. See the [examples](examples/) directory for details.

