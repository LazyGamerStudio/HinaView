# Third-Party Licenses and Notices

This document lists all third-party software and libraries used in HinaView, along with their respective licenses and copyright notices.

---

## Table of Contents

1. [Summary of Licenses](#summary-of-licenses)
2. [Rust Crates (crates.io)](#rust-crates-cratesio)
3. [External Dependencies](#external-dependencies)
4. [System Libraries](#system-libraries)

---

## Summary of Licenses

| License | Count |
|---------|-------|
| Apache-2.0 OR MIT | 338 |
| MIT | 136 |
| Apache-2.0 | 15 |
| Unicode-3.0 | 18 |
| MIT OR Unlicense | 8 |
| BSD-3-Clause | 6 |
| BSD-2-Clause | 3 |
| BSL-1.0 | 2 |
| MPL-2.0 | 2 |
| Zlib | 4 |
| ISC | 3 |
| CC0-1.0 | 1 |
| LGPL-3.0-or-later | 2 |
| Public Domain | 1 |
| BSD-3-Clause (modified) | 1 |

---

## Rust Crates (crates.io)

### Apache-2.0 OR MIT (338 crates)

Major crates include:
- `anyhow`, `async-trait`, `serde`, `serde_json`, `tokio`
- `wgpu`, `winit`, `egui`, `egui-wgpu`, `egui-winit`
- `rayon`, `crossbeam`, `parking_lot`, `lru`
- `reqwest`, `hyper`, `http`, `tower`
- `tracing`, `tracing-subscriber`, `tracing-appender`
- `image`, `png`, `gif`, `tiff`, `exr`, `webp`
- `rusqlite`, `libsqlite3-sys`
- And 300+ additional crates

Full list available in `licenses.json`.

### MIT (136 crates)

Major crates include:
- `bytes`, `tokio`, `tokio-util`, `tokio-native-tls`
- `hyper`, `hyper-util`, `h2`, `http`, `http-body`
- `tower`, `tower-http`, `tower-layer`, `tower-service`
- `tracing`, `tracing-core`, `tracing-attributes`
- `libwebp-sys`, `libsqlite3-sys`
- `zbus`, `zvariant`, `zvariant_derive`
- `objc2`, `objc2-foundation`, `objc2-app-kit`
- `rfd`, `fontdb`, `fontconfig-parser`
- And 100+ additional crates

### Apache-2.0 (15 crates)

- `ab_glyph`, `ab_glyph_rasterizer`, `owned_ttf_parser`
- `clang-sys`, `codespan-reporting`, `gethostname`
- `gl_generator`, `glutin_wgl_sys`, `khronos_api`
- `lzma-rust2`, `openssl`, `spirv`, `sync_wrapper`
- `winit`, `zopfli`

### MIT OR Unlicense (8 crates)

- `aho-corasick`, `byteorder-lite`, `memchr`
- `same-file`, `termcolor`, `turbojpeg-sys`
- `walkdir`, `winapi-util`

### BSD-3-Clause (6 crates)

- `bindgen`, `exr`, `lebe`, `subtle`
- `tiny-skia`, `tiny-skia-path`

### BSD-2-Clause (3 crates)

- `arrayref`, `kamadak-exif`, `mutate_once`

### BSL-1.0 - Boost Software License 1.0 (2 crates)

- `clipboard-win`, `error-code`

### MPL-2.0 - Mozilla Public License 2.0 (2 crates)

- `cbindgen`, `option-ext`

### Zlib (4 crates)

- `foldhash` (2), `slotmap`, `zlib-rs`

### Unicode-3.0 (18 crates)

ICU4X crates:
- `icu_collections`, `icu_locale_core`, `icu_normalizer`
- `icu_normalizer_data`, `icu_properties`, `icu_properties_data`
- `icu_provider`, `litemap`, `potential_utf`, `tinystr`
- `writeable`, `yoke`, `yoke-derive`, `zerofrom`
- `zerofrom-derive`, `zerotrie`, `zerovec`, `zerovec-derive`

### Other Licenses

| License | Crates |
|---------|--------|
| CC0-1.0 | `hexf-parse` |
| CC0-1.0 OR MIT-0 | `ppmd-rust` |
| ISC | `libloading`, `rustls-webpki`, `untrusted` |
| 0BSD OR Apache-2.0 OR MIT | `adler2` |
| Apache-2.0 AND ISC | `ring` |
| Apache-2.0 AND LGPL-2.1-or-later OR MIT | `r-efi` |
| Apache-2.0 AND MIT | `dpi` |
| Apache-2.0 OR BSD-2-Clause OR MIT | `zerocopy`, `zerocopy-derive` |
| Apache-2.0 OR BSD-3-Clause OR MIT | `num_enum`, `num_enum_derive` |
| Apache-2.0 OR BSL-1.0 | `ryu` |
| Apache-2.0 OR CC0-1.0 OR MIT-0 | `constant_time_eq` |
| Apache-2.0 OR ISC OR MIT | `hyper-rustls`, `rustls` |
| Apache-2.0 OR LGPL-2.1-or-later OR MIT | `r-efi` |
| Apache-2.0 OR MIT OR Zlib | `bytemuck`, `bytemuck_derive`, `glow`, etc. |
| bzip2-1.0.6 | `libbz2-rs-sys` |

---

## External Dependencies

### CrabbyAvif
**License:** Apache-2.0  
**Copyright:** Copyright 2024 Google LLC  
**Repository:** https://github.com/google/CrabbyAvif

```
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0
```

### dav1d
**License:** BSD-2-Clause  
**Copyright:** Copyright © 2018-2025, VideoLAN and dav1d authors  
**Repository:** https://code.videolan.org/videolan/dav1d

```
Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED.
```

### libjxl (JPEG XL)
**License:** BSD-3-Clause  
**Copyright:** Copyright (c) the JPEG XL Project Authors  
**Repository:** https://github.com/libjxl/libjxl

```
Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.
3. Neither the name of the copyright holder nor the names of its
   contributors may be used to endorse or promote products derived from
   this software without specific prior written permission.
```

---

## System Libraries

### libheif
**License:** LGPL-3.0-or-later (GNU Lesser General Public License v3.0 or later)  
**Copyright:** Copyright (c) 2017-2025 Dirk Farin  
**Repository:** https://github.com/strukturag/libheif

```
libheif is free software: you can redistribute it and/or modify
it under the terms of the GNU Lesser General Public License as
published by the Free Software Foundation, either version 3 of
the License, or (at your option) any later version.
```

**Notice:** As this software is licensed under LGPL-3.0, you must make available the source code of any modifications to libheif itself. This does not affect the licensing of your own application code that merely links to libheif.

### libde265
**License:** LGPL-3.0-or-later (GNU Lesser General Public License v3.0 or later)  
**Copyright:** Copyright (c) 2013-2014 struktur AG, Dirk Farin  
**Repository:** https://github.com/strukturag/libde265

```
libde265 is free software: you can redistribute it and/or modify
it under the terms of the GNU Lesser General Public License as
published by the Free Software Foundation, either version 3 of
the License, or (at your option) any later version.
```

**Notice:** As this software is licensed under LGPL-3.0, you must make available the source code of any modifications to libde265 itself. This does not affect the licensing of your own application code that merely links to libde265.

### SQLite
**License:** Public Domain  
**Copyright:** None - Dedicated to the public domain  
**Website:** https://www.sqlite.org

```
All code and documentation has been dedicated to the public domain
by the authors. No license is required to use SQLite.
```

---

## Full License Texts

### Apache License 2.0

See: https://www.apache.org/licenses/LICENSE-2.0

### MIT License

```
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### BSD 2-Clause License

```
Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED.
```

### BSD 3-Clause License

```
Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.
3. Neither the name of the copyright holder nor the names of its
   contributors may be used to endorse or promote products derived from
   this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED.
```

### GNU Lesser General Public License v3.0

See: https://www.gnu.org/licenses/lgpl-3.0.html

### Mozilla Public License 2.0

See: https://www.mozilla.org/en-US/MPL/2.0/

### Boost Software License 1.0

```
Boost Software License - Version 1.0 - August 17th, 2003

Permission is hereby granted, free of charge, to any person or organization
obtaining a copy of the software and accompanying documentation covered by
this license (the "Software") to use, reproduce, display, distribute,
execute, and transmit the Software, and to prepare derivative works of the
Software, and to permit third-parties to whom the Software is furnished to
do so, all subject to the following:

The copyright notices in the Software and this entire statement, including
the above license grant, this restriction and the following disclaimer,
must be included in all copies of the Software, in whole or in part, and
all derivative works of the Software, unless such copies or derivative
works are solely in the form of machine-executable object code generated by
a source language processor.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE, TITLE AND NON-INFRINGEMENT. IN NO EVENT
SHALL THE COPYRIGHT HOLDERS OR ANYONE DISTRIBUTING THE SOFTWARE BE LIABLE
FOR ANY DAMAGES OR OTHER LIABILITY, WHETHER IN CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
```

---

## Notes

- This document was generated automatically from `cargo license` output and dependency analysis.
- For the complete list of all dependencies with their exact versions, see `Cargo.lock` and `licenses.json`.
- Some crates offer dual or triple licensing - you may choose any of the listed licenses.
- The LGPL-licensed libraries (libheif, libde265) are dynamically linked, which satisfies LGPL requirements for allowing users to replace these libraries.

---

*Last updated: 2026-03-24*
