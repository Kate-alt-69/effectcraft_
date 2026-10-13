# Community fork workflow (F1)

This fork is based on EffectCraft by the ArtCraft team. Its source and original additions remain
MIT OR Apache-2.0. Restricted ArtCraft wordmarks and header marks have been removed; the header
uses EffectCraft's existing open app icon. Upstream community links remain available.

## Editing workspace

Choose **Editing** in the workspace bar or Window → Workspace to adopt the fork's layout.
A fresh Default layout also uses the fork arrangement. Existing saved Default layouts are kept;
Editing lets existing users try the new arrangement without deleting their layout.

- The left sidebar opens on **Tools**. Every toolbar mode is listed by name, with its shortcut.
  Selecting a tool also updates its toolbar slot. Shape, roto, and puppet tools expose relevant
  options; text and paint tools link to their existing detailed panels.
- **Project** and **Effects & Presets** remain tabs beside Tools. Asset importing and browsing
  still work through the same commands.
- Playback lives beneath the canvas: first/previous frame, play/pause, next/last frame, and Loop.
  The optional Preview panel still provides shortcut-specific preview configuration. Both
  transports share those settings and command handlers.
- Panel and workspace tabs have clear selected backgrounds instead of link-like underlines.
- Numeric property controls have a field background and border. Drag to scrub, click to type,
  Shift for coarse changes, Ctrl/Cmd for fine changes, and Tab/Shift+Tab between inline editors.
  Keyboard focus and drag states remain visible. The existing animation controls remain beside
  each property.
- The Properties Inspector includes an effect stack with bypass, Edit, move up/down, Reset,
  Remove, and Add effect or preset. These actions use the existing undoable engine commands.

## Original effects

All three effects are in the normal registry and available from the UI, commands, scripts, CLI,
and MCP. Parameters animate and accept expressions like other effect properties. They run on
the CPU, including when a GPU composition requires a CPU fallback. GPU acceleration is future work.

| Effect | Stable ID | Controls and behavior |
|---|---|---|
| Animated Mesh Gradient | `ec.generate.meshgradient` | Five point/color pairs, point influence, opacity, preserve source alpha. Smooth inverse-distance color interpolation; animate the points or colors to move the field. Point defaults scale to layer bounds. |
| Distance Outline | `ec.stylize.distanceoutline` | Band width, expansion, band count/spacing, softness, alpha threshold, first/last colors, and compositing. A separable Euclidean distance transform builds contour bands around the thresholded silhouette; transparent padding expands the output bounds. |
| Halftone Print | `ec.stylize.halftoneprint` | Cell size, angle, contrast, dots/lines/squares, ink/paper, and original-image mix. Rotated cells sample source luminance, preserve source transparency, and turn coverage into a print pattern. Mix 100% restores the original. |

These are original implementations, with no claim of reproducing a proprietary plug-in.
They use bounded allocations: buffers above 16,777,216 pixels, invalid buffer geometry, and
outline padding above 4,096 pixels pass through unchanged. The pixel cap applies to the padded
outline buffer as well as the input. This initial cap supports ordinary 1080p/4K work but is a
limitation for larger canvases or unusually large contours. Finite controls are sanitized, and
HDR effect colors are retained within finite half-float range. Outline geometry uses an alpha
threshold rather than tracing fractional alpha boundaries.

These effects do not read the clock directly or depend on adjacent frames. Their evaluated
animated property values drive changing frames, using the existing property/cache machinery.
The larger adaptive-cache scheduler redesign has not been implemented in F1.

## Original presets

F1 adds **40** presets: 12 text reveals, 8 shape motions, 8 animated backgrounds, 6 transitions,
and 6 image looks. Together with the 31 upstream presets, this is **71** bundled presets.

Open **Effects & Presets → Animation Presets** to browse the categories. Double-click a preset
with a suitable layer selected; text presets require text layers. The Inspector's text preset
menu also includes the new text reveals.

Backgrounds apply a mesh-gradient effect to a selected solid or other effect-capable layer;
they do not create layers. Their animated point positions use normalized layer coordinates,
so the same preset works at different layer sizes. Shape motions animate the existing anchor's
scale or rotation with explicit values, so they can replace existing keys on those properties.
Use Undo to restore them, and edit their keyframes afterward for timing and intensity. Image
looks add editable effects; transitions include in/out metadata.

The fork libraries `crates/engine/presets/creative.json` and `text_creative.json` are separate
from the upstream transition and text libraries. An effect
preset can opt into `normalizedPoints: true`; point values and point keyframes are resolved to
each target layer's bounds at application time. Existing pixel-based presets keep their behavior.

## Maintaining the fork

Shared engine commands, property identifiers, upstream effects, and project formats are retained.
The new Tools panel is frontend state. Existing saved dock trees still deserialize by panel name.
New effects and presets are registered through the same extension points used by upstream.

Two existing GPU boundary issues were also corrected during validation: HexTile prepares
f64 sampling coordinates before GPU sampling, and Threshold RGB preserves CPU threshold
decisions near division rounding boundaries. The HexTile map uses eight bytes per pixel,
bounded by the device storage-buffer limit; larger maps use the existing CPU fallback.

Run `cargo xtask ci` before committing. UI changes also need visual inspection using the headless
snapshot example. `Plan.md` records the remaining canvas tools, visual Library, reusable components,
ArtCraft handoff, and caching work.
