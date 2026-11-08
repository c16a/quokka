# Relinking Quokka with a different FFmpeg

The Linux release binary contains FFmpeg 8.1, which is licensed under the GNU Lesser General Public
License, version 2.1 or later. This page explains how to use a different FFmpeg 8.1 build, as the
LGPL requires.

## What is in the binary

The default build compiles FFmpeg into the `quokka` binary. FFmpeg is built from the `release/8.1`
branch at commit `a35157abb96167fd7308edc92b55408d5bb084b0`, with the configure options the
`ffmpeg-sys-next` crate uses. The build has no GPL or non-free parts (`CONFIG_GPL=0`,
`CONFIG_NONFREE=0`, `CONFIG_VERSION3=0`). It includes libavcodec, libavformat and libavutil.

The source for that exact commit is at https://github.com/FFmpeg/FFmpeg/tree/a35157abb96167fd7308edc92b55408d5bb084b0.
The LGPL-2.1 licence text is in `LICENSES/LGPL-2.1.txt`.

## Build against your own FFmpeg

1. Get FFmpeg 8.1 source, either from the commit above or from another 8.1 source tree you have
   changed.
2. Build it as shared libraries, and install it to a prefix of your choice. For example:

   ```bash
   ./configure --enable-shared --disable-static --disable-programs --disable-doc --prefix="$HOME/ffmpeg-8.1"
   make -j"$(nproc)"
   make install
   ```

3. Build Quokka against that FFmpeg, without the bundled copy:

   ```bash
   PKG_CONFIG_PATH="$HOME/ffmpeg-8.1/lib/pkgconfig" cargo build -p quokka-app --release --no-default-features
   ```

   `just release-system` runs the same build with the system's FFmpeg.

4. Run it with the shared libraries on the library path:

   ```bash
   LD_LIBRARY_PATH="$HOME/ffmpeg-8.1/lib" ./target/release/quokka
   ```

## Requirements

- FFmpeg 8.1 development files, found through `pkg-config`.
- `clang`, for the bindings.

The system build needs the FFmpeg major version to match the bindings: 8.x. It has not been tested
against a distribution's FFmpeg packages yet.

## Modified FFmpeg

If you change FFmpeg, the LGPL requires you to provide the source of your changes to anyone you give
the binary to.
