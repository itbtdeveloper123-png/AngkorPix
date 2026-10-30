# AngkorPix

<p align="center">
  <img src="assets/app-icon/hicolor/256x256/apps/ai.storyteller.photocraft.png" width="128" height="128" alt="AngkorPix icon: an engraved nine-tailed fox on a blue tile">
</p>


<p align="center">
  <b>High-performance, native image editor written in pure Rust.</b><br>
  Layers, masks, adjustment layers, layer styles, typography, vectors, brushes, and full PSD/PSB compatibility.<br>
  Native, fast, offline, and completely open source.
</p>

<p align="center">
  <img alt="100% Rust" src="https://img.shields.io/badge/100%25-Rust-b7410e?style=flat-square&logo=rust">
  <img alt="macOS · Windows · Linux · FreeBSD · Web" src="https://img.shields.io/badge/macOS%20%C2%B7%20Windows%20%C2%B7%20Linux%20%C2%B7%20FreeBSD%20%C2%B7%20Web-native-2f7bf5?style=flat-square">
  <img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-3a3a3a?style=flat-square">
  <a href="https://github.com/itbtdeveloper123-png/AngkorPix"><img alt="GitHub Repo" src="https://img.shields.io/badge/GitHub-AngkorPix-181717?style=flat-square&logo=github"></a>
  <img alt="Status: early alpha" src="https://img.shields.io/badge/status-early%20alpha-d69e2e?style=flat-square">
</p>

<p align="center">
  <a href="https://github.com/itbtdeveloper123-png/AngkorPix/releases"><b>Download AngkorPix</b></a> ·
  <a href="https://github.com/itbtdeveloper123-png/ArtPixels">ArtPixels Suite</a> ·
  <a href="https://github.com/itbtdeveloper123-png/AngkorVector">AngkorVector</a>
</p>

<br>

<p align="center">
  <img src="docs/images/photocraft-demo.jpg" alt="AngkorPix editing Hokusai's The Great Wave: caption card with a drop shadow, Title and Credit type layers, Vibrance and Curves adjustment layers, and the Curves editor drawn over the image's histogram" width="100%">
  <br>
  <sub>A caption card with a drop shadow, live type, and Vibrance and Curves adjustment layers, with the Curves editor open.<br>
  <i>The Great Wave off Kanagawa</i>, Katsushika Hokusai, c. 1831</sub>
</p>

<p align="center">
  <a href="#features">Features</a> ·
  <a href="#everything-in-the-box">Everything in the box</a> ·
  <a href="#psd-without-compromise">PSD</a> ·
  <a href="#built-for-automation">Automation</a> ·
  <a href="#under-the-hood">Under the hood</a> ·
  <a href="#get-started">Get started</a> ·
  <a href="#license-and-credits">License</a>
</p>

<br>

<table>
  <tr>
    <td width="25%" valign="top">
      <h3>🎛️ Familiar by design</h3>
      The menus, shortcuts, panels and tools are where your hands expect them, from ⌘J to ⇧⌘D. Designed for seamless, professional image editing workflows.
    </td>
    <td width="25%" valign="top">
      <h3>⚡ Native and fast</h3>
      A hardware-accelerated GPU compositor on wgpu (Metal, Vulkan, DX12, WebGPU), copy-on-write tiles and multithreaded filters. No Electron, no web views, zero bloat.
    </td>
    <td width="25%" valign="top">
      <h3>🗂️ Real PSD files</h3>
      Open, edit and save layered Photoshop documents. Re-saving preserves layer structures, blend modes, adjustments, and styles with high fidelity.
    </td>
    <td width="25%" valign="top">
      <h3>🤖 Automation & MCP Ready</h3>
      Every action is a command, allowing control from the UI, CLI, JSON control channel, or Model Context Protocol (MCP) AI agents.
    </td>
  </tr>
</table>

<br>

## Features

Every screenshot here is the real app at work on public-domain art, rendered offscreen through its control channel.

<table>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/images/photocraft-adjustments.jpg" alt="Levels and Vibrance adjustment layers with Histogram panel" width="100%">
      <br>
      <sub>Levels and Vibrance adjustment layers, with the live Histogram panel.<br><i>Impression, Sunrise</i>, Claude Monet, 1872</sub>
      <h3>Edit without regret</h3>
      Adjustment layers keep every edit live. Stack Levels, Curves, Vibrance, Hue/Saturation and a dozen more, mask them to an area, reorder them, or turn them off, and your original pixels never change.
      <br><br>
      <b>16 adjustment layers</b> that also apply directly to pixels, including Curves with per-channel editing, Levels with a live histogram, Black &amp; White, Channel Mixer, Gradient Map, Photo Filter, Selective Color and Color Lookup (.cube, .3dl, .look).
    </td>
    <td width="50%" valign="top">
      <img src="docs/images/photocraft-layer-styles.jpg" alt="Outer Glow and Stroke in Layer Style dialog" width="100%">
      <br>
      <sub>Outer Glow and Stroke on a live type layer, in the Layer Style dialog.<br><i>Earthrise</i>, William Anders / NASA, 1968</sub>
      <h3>Styles that sell the shot</h3>
      Drop Shadow, Inner Shadow, Outer and Inner Glow, Bevel &amp; Emboss, Satin, Stroke, and Color, Gradient and Pattern Overlay, live on any layer, including type. Patterns come from a library (built-ins, Edit › Define Pattern, <code>.pat</code> import/export) and PSD <code>Patt</code> blocks.
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/images/photocraft-masks.jpg" alt="Hue/Saturation layer with an elliptical mask" width="100%">
      <br>
      <sub>An elliptical selection becomes the mask of a Hue/Saturation layer.<br><i>Girl with a Pearl Earring</i>, Johannes Vermeer, c. 1665</sub>
      <h3>Selections that understand your image</h3>
      Marquees, lassos and the Magic Wand for precision; Quick Selection, Object Selection and Select Subject when you want the computer to do the tracing; Select and Mask to refine hair-fine edges.
    </td>
    <td width="50%" valign="top">
      <img src="docs/images/photocraft-type.jpg" alt="Headline and paragraph text editing" width="100%">
      <br>
      <sub>A headline edited in place, with a byline and a paragraph of body text.<br><i>Among the Sierra Nevada, California</i>, Albert Bierstadt, 1868</sub>
      <h3>Type that sets beautifully</h3>
      Point and paragraph text, edited right on the canvas, with full Character and Paragraph controls: font, weight, size, leading, tracking, alignment and colour.
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/images/photocraft-vector.jpg" alt="Shape layers and vector paths" width="100%">
      <br>
      <sub>A badge made of shape layers: a gradient-filled lotus, a star and a dotted ring.<br><i>Water Lilies</i>, Claude Monet, 1906</sub>
      <h3>Pixel-perfect vectors</h3>
      Rectangle, Ellipse, Triangle, Polygon, Line and the Pen, Freeform Pen and Curvature Pen tools, with resolution-independent shape layers, gradient fills, and dashed, aligned strokes.
    </td>
    <td width="50%" valign="top">
      <img src="docs/images/photocraft-filters.jpg" alt="Twirl filter preview" width="100%">
      <br>
      <sub>Twirl previews live on the canvas, only inside the selection.<br><i>The Starry Night</i>, Vincent van Gogh, 1889</sub>
      <h3>See it before you commit</h3>
      Every filter dialog previews live on the canvas, through your selection. Blurs (Gaussian, Box, Motion, Radial, Surface, Smart, Lens, Shape), sharpening, Reduce Noise, distortions, Pixelate, Stylize, and Render effects.
    </td>
  </tr>
</table>

<br>

## Everything in the box

<table>
  <tr>
    <td width="33%" valign="top">
      <h4>🧰 68 tools</h4>
      Move · Artboard · Rectangular and Elliptical Marquee · Single Row Marquee · Single Column Marquee · Lasso · Polygonal Lasso · Magnetic Lasso · Object Selection · Quick Selection · Magic Wand · Crop · Perspective Crop · Slice · Slice Select · Frame · Eyedropper · Color Sampler · Ruler · Note · Count · Remove · Spot Healing Brush · Healing Brush · Patch · Content-Aware Move · Red Eye · Brush · Pencil · Mixer Brush · Color Replacement · Clone Stamp · Pattern Stamp · History Brush · Art History Brush · Eraser · Background Eraser · Magic Eraser · Gradient · Paint Bucket · Blur · Sharpen · Smudge · Dodge · Burn · Sponge · Pen · Freeform Pen · Curvature Pen · Add Anchor Point · Delete Anchor Point · Convert Point · Horizontal and Vertical Type · Horizontal and Vertical Type Mask · Path Selection · Direct Selection · Rectangle · Ellipse · Triangle · Polygon · Line · Custom Shape · Hand · Rotate View · Zoom
    </td>
    <td width="33%" valign="top">
      <h4>🖌️ Real Brush Engine</h4>
      Shape Dynamics, Scattering, Texture, Dual Brush, Color Dynamics, Transfer, Wet Edges, Build-up, Smoothing with pen pressure, tilt, rotation and direction.
    </td>
    <td width="33%" valign="top">
      <h4>🗃️ Layers Done Properly</h4>
      Groups, clipping masks, pixel and vector masks, fill layers, adjustment layers, smart objects, multi-layer selection with align/distribute, and 27 blend modes.
    </td>
  </tr>
  <tr>
    <td width="33%" valign="top">
      <h4>🎨 Any Colour, Any Depth</h4>
      RGB, Grayscale, CMYK and Lab documents at 8, 16 and 32 bits per channel. Real ICC colour management in pure Rust: embedded profiles, Assign/Convert, soft proofing.
    </td>
    <td width="33%" valign="top">
      <h4>🗂️ Formats</h4>
      PSD and PSB, layered TIFF (Photoshop's layer data in the TIFF, read and written in either byte order), <a href="docs/pdn.md">Paint.NET PDN3 (read, with editable layers and all 14 blend modes)</a>, <a href="docs/ora.md">OpenRaster (read and write, with layers, groups and blend modes, for Krita, MyPaint and GIMP)</a>, <a href="docs/xcf.md">GIMP XCF (read, with layers, groups, masks, blend modes, channels and the selection)</a>, <a href="docs/kra.md">Krita .kra (read, with paint layers, groups, blend modes and transparency masks)</a>, SVG (opens as shape layers, places as a vector Smart Object), plus flat PNG, JPEG, TIFF, WebP (lossy and lossless), GIF, BMP, TGA, ICO, QOI, PNM, OpenEXR, Radiance HDR and FITS (astronomy images: 8/16-bit and 32-bit float, gray or RGB, header kept), and the native <code>.pcraft</code> format. Flat formats retain the depths they support: PNG keeps 8/16-bit samples, TIFF keeps 8/16-bit and 32-bit float samples, and OpenEXR keeps half/float samples; JPEG, WebP and the other 8-bit formats reduce higher-depth samples with an export warning. See the <a href="crates/codecs/README.md#capability-matrix-default-build">codec capability matrix</a> for each format. HEIC photos from iPhone and Mac are read-only (included in official builds; optional <code>--features heif</code> for source builds). AVIF is write-only with the optional <code>avif</code> build feature; AVIF decoding and JPEG XL are unavailable. Affinity documents (<code>.af</code>, <code>.afdesign</code>, <code>.afphoto</code>, <code>.afpub</code>) open natively, read only, with artboards, layers, vectors, text, images and masks; effects, adjustments and other unsupported parts are listed in a warning, and Affinity export is unavailable.
    </td>
    <td width="33%" valign="top">
      <h4>🪄 Everyday Essentials</h4>
      Auto Tone/Contrast/Color, Image/Canvas Size, Crop &amp; Trim, Reveal All, Fill &amp; Stroke, Guides, Snapping, Actions record &amp; replay, and Command Palette.
    </td>
  </tr>
</table>

<br>

PhotoCraft/AngkorPix provides robust, clean-room PSD support tested against a corpus of real-world files:

- **Faithful round trips:** opening and re-saving a document renders the same for 307 of the 309 files in the psd-tools test set and 169 of 170 in our mixed ag-psd/psd-tools set (`crates/io/tests/it/corpus.rs`), and anything we don't model yet (raw blocks, descriptors, extras) is carried over instead of being dropped.
- **Matching pixels:** a composite oracle compares our render with Photoshop's own merged image, covering gradient interpolation (Classic, Perceptual and Linear), layer effects, shape strokes, clipping and fill opacity.
- **Large canvases:** PSB support, 16 and 32-bit floating point buffers, and CMYK/Lab spaces open natively.


## Built for agents

Every menu item, tool and dialog runs a command from one registry of 500+ commands. The UI, the CLI, the JSON control channel and the MCP server all call the same commands, so anything you can click, a script or an AI agent can do too.

```sh
# Headless: open, edit, save
photocraft-cli run wave.psd \
  --cmd filter.sharpen.smartSharpen     --params '{"amount":80}' \
  --cmd layer.newAdjustmentLayer.curves --params '{"points":[[0,0],[64,48],[192,212],[255,255]]}' \
  --out wave-final.png

# Apply one action list to a folder of images
photocraft-cli batch --actions grade.json --in ./raw --out ./graded

# Every subcommand explains itself
photocraft-cli batch --help

# Let an agent drive it over MCP (headless, or bridged to the running app)
photocraft-cli mcp
```

The desktop app also offers an authenticated, loopback-only control channel (`photocraft --control`) for inspecting UI state, driving tools with pointer events, and taking offscreen screenshots. Every image in this README was rendered that way. See [`docs/control-protocol.md`](docs/control-protocol.md).

## Under the hood

- **Engine first:** a pure-data document model and a command engine, with a thin egui UI on top. Layering is enforced at build time across 24 crates.
- **Two compositors:** a CPU compositor serves as the reference oracle, and a wgpu compositor puts the canvas on the GPU. They are tested against each other.
- **Copy-on-write tiles:** 256² sparse tiles make undo cheap and huge canvases light, and effect maps are cached per layer state.
- **Runs in the browser:** the whole engine and UI compile to WebAssembly.
- **Clean-room:** implemented from public specs and observed behaviour only. No proprietary code, shaders or assets.
- **Tested:** more than 1,700 tests, including PSD round trips, synthetic generators, compositor oracles and multi-depth checks.

## Get started

```sh
git clone https://github.com/storytold/photocraft
cd photocraft
cargo run --release -p photocraft -- image.psd   # the desktop app
cargo test --workspace                           # the test suite
```

Japanese fonts for the UI and Type tool come from [craft-fonts](https://github.com/storytold/craft-fonts), an optional build input (desktop release builds always include it). Without it PhotoCraft uses your system's CJK fonts:

```sh
git clone https://github.com/storytold/craft-fonts ../craft-fonts
CRAFT_FONTS_DIR="$PWD/../craft-fonts" cargo run --release -p photocraft
```

New contributors and AI agents: start with [`AGENTS.md`](AGENTS.md), then [`docs/`](docs/).

Installers for macOS, Windows, Linux, FreeBSD and the web are attached to each [GitHub release](https://github.com/storytold/photocraft/releases). On Linux you can pick an AppImage, a `.deb`, an `.rpm`, a tarball or a Flatpak bundle. The bundle needs the freedesktop runtime from [Flathub](https://flathub.org/setup), which `flatpak` offers to install along with it:

```sh
flatpak install --user photocraft-<version>-linux-x86_64.flatpak   # or -linux-aarch64
flatpak run ai.storyteller.photocraft
```

The Flatpak prints through the host's CUPS socket (File › Print talks IPP to it directly, no `lp` needed); without CUPS, or without a printer, File › Print opens the system print dialog through the print portal. See [printing](docs/printing.md).

The AppImage needs no install: the first run registers its launcher icon and menu entry in `~/.local/share` so the dock shows PhotoCraft's icon on Wayland. Set `PHOTOCRAFT_NO_DESKTOP_INTEGRATION=1` to skip that, and see [`docs/releasing.md`](docs/releasing.md) › Linux to undo it.

On a Wayland session, files dropped on the window don't open yet: winit 0.30, the windowing library under egui, has no Wayland drag and drop (#386). Use File › Open, or copy the image in your file manager and paste it with Ctrl+V. To drag and drop, start PhotoCraft under XWayland: `WAYLAND_DISPLAY= photocraft`, `WAYLAND_DISPLAY= ./photocraft-<version>-linux-x86_64.AppImage`, or `flatpak run --nosocket=wayland --socket=x11 ai.storyteller.photocraft`. On Wayland, PhotoCraft shows the command for your install in a notice until you dismiss it. Preferences › Performance › Linux display server = X11 does the same on every launch. With a fractional display scale (125%, 150%, 175%) an XWayland window is drawn at a lower resolution and enlarged by the compositor, so the interface is blurred there; native Wayland stays sharp (#1072).

On macOS, the command-line tool comes as `photocraft-cli-<version>-macos-universal.zip`. The binary is signed with the same Developer ID as the app and notarized by Apple. A bare binary can't carry a stapled notarization ticket the way the DMG does, so the first time you run it macOS checks the notarization online. You can confirm it yourself:

```sh
ditto -x -k photocraft-cli-<version>-macos-universal.zip .
spctl --assess --type install -vv photocraft-cli-<version>-macos-universal/photocraft-cli
# ... accepted, source=Notarized Developer ID
```

On FreeBSD 14 (x86_64), the release has a tarball laid out like `/usr/local`. Install the runtime libraries, then unpack it there:

```sh
pkg install libxkbcommon wayland libX11 libXcursor libXrandr libXi libxcb mesa-libs vulkan-loader gtk3 fontconfig freetype2 alsa-lib
tar -xzf photocraft-<version>-freebsd-x86_64.tar.gz --strip-components 1 -C /usr/local
photocraft
```

Maintainers: [`docs/releasing.md`](docs/releasing.md) explains how releases are built, signed and published.

> [!IMPORTANT]
> **Status:** PhotoCraft is in early alpha, and we want to be straight about where it stands: much of Photoshop's feature surface exists in some form, but **it is not yet a Photoshop replacement for daily professional work**. The biggest gaps are AI/generative features, 15 missing tools, depth in typography and pro workflows, and plug-in compatibility. Every Photoshop menu item is wired to a command ([`docs/parity-checklist.md`](docs/parity-checklist.md)), but that measures wiring, not behaviour. The honest, dimension-by-dimension picture and where we're going next are in the [roadmap](ROADMAP.md) and [parity assessment](docs/target-app-parity.md). Expect rough edges, and please file issues (include your OS, document size, layer count and a screenshot). You can also tell us what broke on [Discord](https://discord.gg/artcraft).

## Community packages

These installers are maintained by community members, not by the PhotoCraft team.

On Gentoo, the [::snakebyte overlay](https://github.com/switch87/snakebyte-overlay) packages the Linux release as `media-gfx/photocraft-bin`:

```sh
eselect repository add snakebyte git https://github.com/switch87/snakebyte-overlay.git
emaint sync -r snakebyte
echo 'media-gfx/photocraft-bin ~amd64' >> /etc/portage/package.accept_keywords/photocraft
emerge --ask media-gfx/photocraft-bin
```

## Documentation

Developer, architecture, automation, format, and security documentation is maintained in the [PhotoCraft documentation book](book/).

To self-host the browser version with Docker, see [Docker web hosting](packaging/web/README.md#docker).

## Security

Security architecture, threat modeling, parser hardening, fuzzing, and vulnerability reporting are covered in the [security documentation](book/src/security/) and the repository [security policy](SECURITY.md).

## Test corpora

PhotoCraft is tested against real files: our own Photoshop-authored oracle PSDs in
[photocraft-corpus](https://github.com/storytold/photocraft-corpus) plus the psd-tools, ag-psd and PngSuite sets, pinned and
sha256-verified. Fetch them with `cargo xtask corpus --all` and run the tests with
`cargo xtask test-corpus` (details in [docs/development.md](docs/development.md#test-corpora)).

## Downloads

**New to PhotoCraft?** Download it from the [PhotoCraft page on getartcraft.com](https://getartcraft.com/apps/photocraft). That's the easiest way to install it.

**Want a specific build or format?** On GitHub, the [latest release](https://github.com/storytold/photocraft/releases/latest) has every build listed below, and [all releases](https://github.com/storytold/photocraft/releases) have earlier versions and their notes. `<ver>` in the file names is the version number, and `SHA256SUMS.txt` lists a checksum for every file.

### Windows

| Build | Installer | Portable |
|---|---|---|
| x64 (64-bit Intel/AMD) | `photocraft-<ver>-windows-x64.msi` | `photocraft-<ver>-windows-x64-portable.zip` |
| arm64 (Snapdragon and other ARM PCs) | `photocraft-<ver>-windows-arm64.msi` | `photocraft-<ver>-windows-arm64-portable.zip` |
| x86 (32-bit) | `photocraft-<ver>-windows-x86.msi` | `photocraft-<ver>-windows-x86-portable.zip` |

Installers and executables are code-signed.

### macOS

| Build | File | Notes |
|---|---|---|
| App, universal (Apple silicon + Intel) | `photocraft-<ver>-macos-universal.dmg` | Signed and notarized |
| Command-line tool, universal | `photocraft-cli-<ver>-macos-universal.zip` | Signed and notarized |

### Linux

| Format | x86_64 | aarch64 (ARM64) | Notes |
|---|---|---|---|
| AppImage | `photocraft-<ver>-linux-x86_64.AppImage` | `photocraft-<ver>-linux-aarch64.AppImage` | Runs anywhere; updates itself with [AppImageUpdate](https://github.com/AppImageCommunity/AppImageUpdate) (`.zsync` files) |
| Flatpak | `photocraft-<ver>-linux-x86_64.flatpak` | `photocraft-<ver>-linux-aarch64.flatpak` | Sandboxed; `flatpak install --user <file>` |
| Debian/Ubuntu | `photocraft-<ver>-linux-x86_64.deb` | `photocraft-<ver>-linux-aarch64.deb` | |
| Fedora/RHEL/openSUSE | `photocraft-<ver>-linux-x86_64.rpm` | `photocraft-<ver>-linux-aarch64.rpm` | |
| Tarball | `photocraft-<ver>-linux-x86_64.tar.gz` | `photocraft-<ver>-linux-aarch64.tar.gz` | Unpack anywhere |

RISC-V (riscv64): `photocraft-<ver>-linux-riscv64.tar.gz` only, built against Ubuntu 24.04 (needs glibc 2.39+).

### FreeBSD

| Build | File |
|---|---|
| x86_64 | `photocraft-<ver>-freebsd-x86_64.tar.gz` |

### Web (WebAssembly)

| Build | File | Notes |
|---|---|---|
| Static site | `photocraft-web-<ver>.zip` | Runs in a modern browser; host it on any static server |

## The Crafting Apps

PhotoCraft is one of the **Crafting Apps**: free, open-source creative tools from the
[ArtCraft](https://getartcraft.com/) team, each written from scratch in Rust and each able to
stand on its own.

| | App | What it's for | Code | Learn more |
|:-:|---|---|---|---|
| <img src="https://raw.githubusercontent.com/storytold/photocraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.photocraft.png" alt="" width="32" height="32"> | **PhotoCraft** | **Image editing: layers, masks, type and real PSD files · you are here** | [GitHub](https://github.com/storytold/photocraft) | [Website](https://getartcraft.com/apps/photocraft) |
| <img src="https://raw.githubusercontent.com/storytold/vectorcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.vectorcraft.png" alt="" width="32" height="32"> | **VectorCraft** | Vector illustration | [GitHub](https://github.com/storytold/vectorcraft) | [Website](https://getartcraft.com/apps/vectorcraft) |
| <img src="https://raw.githubusercontent.com/storytold/filmcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.filmcraft.png" alt="" width="32" height="32"> | **FilmCraft** | Video editing, color and sound | [GitHub](https://github.com/storytold/filmcraft) | [Website](https://getartcraft.com/apps/filmcraft) |
| <img src="https://raw.githubusercontent.com/storytold/lightcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.lightcraft.png" alt="" width="32" height="32"> | **LightCraft** | Photo library and raw development | [GitHub](https://github.com/storytold/lightcraft) | [Website](https://getartcraft.com/apps/lightcraft) |
| <img src="https://raw.githubusercontent.com/storytold/pdfcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.pdfcraft.png" alt="" width="32" height="32"> | **PdfCraft** | Reading, organizing and protecting PDFs | [GitHub](https://github.com/storytold/pdfcraft) | [Website](https://getartcraft.com/apps/pdfcraft) |
| <img src="https://raw.githubusercontent.com/storytold/effectcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.effectcraft.png" alt="" width="32" height="32"> | **EffectCraft** | Motion graphics and visual effects | [GitHub](https://github.com/storytold/effectcraft) | [Website](https://getartcraft.com/apps/effectcraft) |
| <img src="https://raw.githubusercontent.com/storytold/designcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.designcraft.png" alt="" width="32" height="32"> | **DesignCraft** | Page layout and publishing | [GitHub](https://github.com/storytold/designcraft) | [Website](https://getartcraft.com/apps/designcraft) |

And [**ArtCraft**](https://getartcraft.com/) itself, our AI image and video studio for artists who want real control.

The Crafting Apps share the same conventions: clean-room and pure Rust, native on macOS, Windows and Linux, in the browser via WebAssembly, and fully drivable by agents.


<br>

## Built for automation

Every menu item, tool, and dialog runs a command from a registry of over 700 commands. The UI, CLI, JSON control channel, and MCP server all call the exact same commands.

```sh
# Headless CLI: open, edit, save
photocraft-cli run input.psd \
  --cmd filter.sharpen.smartSharpen     --params '{"amount":80}' \
  --cmd layer.newAdjustmentLayer.curves --params '{"points":[[0,0],[64,48],[192,212],[255,255]]}' \
  --out output.png

# Batch processing
photocraft-cli batch --actions grade.json --in ./raw --out ./graded

# Drive via Model Context Protocol (MCP) for AI agent integration
photocraft-cli mcp
```

<br>

## Under the hood

- **Engine-first architecture:** Pure-data document model and command registry, with a thin native egui/eframe UI.
- **Dual compositors:** A deterministic CPU reference compositor and an accelerated wgpu GPU compositor.
- **Tile-based Copy-On-Write:** 256×256 sparse tiles make undo/redo instantaneous and large canvases memory-efficient.
- **Pure safe Rust:** Zero webview shells (no Electron, no Tauri), cross-platform, compiles to desktop and WebAssembly.

<br>

## Get started

### Prerequisites
- Rust stable (1.95+): [https://rustup.rs](https://rustup.rs)

### Build and Run
```sh
# Clone the repository
git clone https://github.com/itbtdeveloper123-png/AngkorPix.git
cd AngkorPix

# Run the desktop application
cargo run -p angkorpix

# Or run in release mode for maximum performance
cargo run --release -p angkorpix
```

<br>

## License and credits

AngkorPix is open source and dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
Based on the foundational work of the PhotoCraft open-source project.

<sub>Adobe and Photoshop are trademarks or registered trademarks of Adobe Inc. AngkorPix is an independent, open-source project and is not affiliated with, sponsored by, or endorsed by Adobe Inc.</sub>
