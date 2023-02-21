# rust_fltk_skia_gl_cpu_demo

rust fltk with skia, if gl mode init failed, it will fallback to soft framebuffer render

tested both in mac and windows

run in OpenGL mode first, if gl init failed, it will fallback to cpu mode

`cargo run`

use this feature to force run in cpu mode

`cargo run --features=force_cpu`

