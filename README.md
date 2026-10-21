# AngkorPix

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
  <br>
  <a href="https://sonarcloud.io/project/overview?id=storytold_photocraft"><img alt="Scanned by SonarCloud" src="https://img.shields.io/badge/Scanned_by-SonarCloud-4b9fd5?style=flat-square&logo=sonarqubecloud&logoColor=white"></a>
  <a href="https://coderabbit.ai"><img alt="Reviewed by CodeRabbit" src="https://img.shields.io/coderabbit/prs/github/storytold/photocraft?style=flat-square&label=Reviewed%20by%20CodeRabbit&logo=coderabbit"></a>
</p>

<p align="center">
  <a href="https://discord.gg/artcraft"><img alt="Join the ArtCraft community on Discord" src="https://img.shields.io/badge/Join%20us%20on%20Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white" height="40"></a>
</p>

<p align="center">
  <a href="https://getartcraft.com/apps/photocraft"><b>PhotoCraft on getartcraft.com</b></a> ·
  <a href="https://getartcraft.com/">ArtCraft</a> ·
  <a href="https://getartcraft.com/apps">All Crafting Apps</a>
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
      <h4>🧰 67 tools</h4>
      Move · Artboard · Rectangular and Elliptical Marquee · Single Row Marquee · Single Column Marquee · Lasso · Polygonal Lasso · Magnetic Lasso · Object Selection · Quick Selection · Magic Wand · Crop · Perspective Crop · Slice · Slice Select · Frame · Eyedropper · Color Sampler · Ruler · Note · Count · Remove · Spot Healing Brush · Healing Brush · Patch · Content-Aware Move · Red Eye · Brush · Pencil · Mixer Brush · Color Replacement · Clone Stamp · Pattern Stamp · History Brush · Eraser · Background Eraser · Magic Eraser · Gradient · Paint Bucket · Blur · Sharpen · Smudge · Dodge · Burn · Sponge · Pen · Freeform Pen · Curvature Pen · Add Anchor Point · Delete Anchor Point · Convert Point · Horizontal and Vertical Type · Horizontal and Vertical Type Mask · Path Selection · Direct Selection · Rectangle · Ellipse · Triangle · Polygon · Line · Custom Shape · Hand · Rotate View · Zoom
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
      <h4>🗂️ Universal Formats</h4>
      PSD &amp; PSB, layered TIFF, OpenRaster (ORA), Paint.NET (PDN), Krita (KRA), SVG, flat PNG, JPEG, TIFF, WebP, OpenEXR, HDR, and native <code>.pcraft</code> format.
    </td>
    <td width="33%" valign="top">
      <h4>🪄 Everyday Essentials</h4>
      Auto Tone/Contrast/Color, Image/Canvas Size, Crop &amp; Trim, Reveal All, Fill &amp; Stroke, Guides, Snapping, Actions record &amp; replay, and Command Palette.
    </td>
  </tr>
</table>

<br>

## PSD without compromise

AngkorPix provides robust, clean-room PSD support tested against a corpus of real-world Photoshop files:
- **Faithful round trips:** Opening and re-saving documents faithfully preserves layers, masks, adjustment parameters, and metadata.
- **Matching pixels:** Composite oracle tests ensure gradient interpolation, layer effects, shape strokes, and blending match Photoshop's output.
- **Large canvases:** PSB support, 16 and 32-bit floating point buffers, and CMYK/Lab spaces open natively.

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
