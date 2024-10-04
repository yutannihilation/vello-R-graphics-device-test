A Graphics Device via gRPC
==========================

This is a prototype for Rust-implemented R graphics device. This relies on these crates:

* winit: for managing a window
* vello: for drawing shapes
* tonic: for communicationg the server and the client with gRPC

Why is tonic needed here? This is because

* winit requires to be executed on the main thread.
* So, if I want to create an interactive device, which runs in background, it
  needs to be a seprated process.
* Since the device runs on a separated process, R needs some measure to
  communicate with the server. tonic enables this.

# Control flow

As described above, the main thread is used by winit (`EventLoop`). tonic also
needs to be run on the main thread (with `#[tokio::main]`), so it's spawned in
the main thread before the winit blocks.

`EventLoop` itself is `!Send` and `!Sync` while tonic requires `Send` and `Sync`
to make the resource accessible via the server. winit provides `EventLoopProxy`
for such cases. A proxy is `Send` and `Sync` and allows to send user-defined
events. The emitted event will be handled in
`ApplicationHandler<T>::user_event()`.

```
┌─────────┐             
│ device  │             
└───▲────┘             
     │                  
     │ winit & vello API
┌────┴────┐             
│eventloop│             
└──▲─────┘             
    │                   
    │ proxy             
┌───┴─────┐             
│ server  │             
└───▲────┘             
     │                  
     │ gRPC             
┌────┴────┐             
│ client  │             
└─────────┘             
```

# R Graphics Device API

cf. <https://github.com/r-devel/r-svn/blob/main/src/include/R_ext/GraphicsDevice.h>

* `activate`: do nothing
* `circle`: draw [`kurbo::Circle`](https://docs.rs/kurbo/latest/kurbo/struct.Circle.html)
* `clip`: [`vello::Scene::push_layer()`](https://docs.rs/vello/latest/vello/struct.Scene.html#method.push_layer) seems to handle this.
* `close`: `event_loop.exit()`
* `deactivate`: do nothing
* `locator`: TBD
* `line`: draw [`kurbo::Line`](https://docs.rs/kurbo/latest/kurbo/struct.Line.html)
* `metricInfo`: TODO
* `mode`:
* `newPage`:
* `polygon`:
* `polyline`:
* `rect`:
* `path`:
* `raster`:
* `cap`:
* `size`:
* `strWidth`:
* `text`:
* `onExit`:
* `newFrameConfirm`:
* `textUTF8`:
* `eventHelper`:
* `holdflush`:
* `setPattern`:
* `releasePattern`:
* `setClipPath`:
* `releaseClipPath`:
* `setMask`:
* `releaseMask`:
* `defineGroup`:
* `useGroup`:
* `releaseGroup`:
* `stroke`:
* `fill`:
* `fillStroke`:
* `capabilities`:
* `glyph`:
