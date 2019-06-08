Fracti
===

This is an implementation of a Barnsley IFS for generating plant like fractals. 

At the moment only fern and maple leaf are built in. Others can be done if you know the coefficients and probabilities. Any other kind of fractal will require a different implementation.

This is a simple project aimed entirely at learning Rust

Requires SDL2 to build.

Running
---
```
cargo run --release
```

Controls:
---
* Escape: Exits
* Left Arrow: decrease GREEN brightness
* Right Arrow: increase GREEN brightness
* Up Arrow: Zoom in
* Down Arrow: Zoom out

