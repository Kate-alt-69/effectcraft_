# EffectCraft fork — living plan

Last updated: 2026-10-12 (Asia/Kolkata)

This file records the discussion, decisions, proposals, evidence, and next steps. Keep it updated as the design develops. Recommendations are proposals until accepted or implemented. Implementation of the first batch (F1) is complete; the original proposals remain recorded below.

## Goal and scope

Upgrade the user's EffectCraft fork into a richer motion graphics and compositing workspace that fits their ArtCraft workflow. Improve the editing experience and add useful tools, original effects, and presets while continuing to receive upstream features. After Effects is a capability reference; the fork should have its own coherent interface.

The exact ArtCraft-to-EffectCraft handoff and the user's desired branding remain to be specified. Do not infer undocumented ArtCraft requirements or assume its Claude development workflow requires Claude specifically.

## Direction agreed in conversation

- Replace ambiguous underlined text controls with recognizable fields, buttons, toggles, choices, and tabs.
- Make the right Inspector contextual, interactive, and useful for the selected layer.
- Move playback controls beside the composition canvas, preferably centered below it.
- Rework the upper-left area around editing tools; keep project assets accessible through a collapsible Assets view.
- Improve preview caching around the playhead with responsive feedback and bounded background work.
- Put the newer interface over shared engine capabilities so upstream additions remain accessible.
- Maintain this Plan.md throughout the discussion.
- Current discussion: audit existing tools/effects, compare with After Effects, and choose original additions compatible with the project's contributor rules.

## Interface and workspace proposals

### Consistent controls

- Numeric fields: units, drag-to-scrub, click-to-type, modifier precision, individual reset, and simple arithmetic where supported.
- Actions: labeled buttons; choices: dropdowns or segmented controls; toggles: explicit state.
- Tabs: visible selected state and adequate hit area.
- Separate animation controls from values: keyframe toggle, previous/next keyframe, easing, and optional curve preview.
- Hover, focus, pressed, disabled, and mixed-selection states; keyboard navigation and shortcuts.
- Searchable commands and properties; explanatory tooltips with shortcuts.
- Consistent spacing, contrast, field sizes, icons, selection colors, and panel behavior. Avoid decorative movement that distracts from the composition.

### Inspector

- Selection-aware views for text, shapes, footage, effects, and composition settings when no layer is selected.
- Editable effect stack: reorder, bypass, before/after, parameter search, save combinations.
- Text: editable content, selected-text font previews, size, tracking, leading, fill/stroke, paragraph layout, animator controls, motion presets.
- Multi-selection: edit shared properties, represent mixed values, align and distribute.
- Pin frequently used controls at the top, including exposed component properties.
- Link property selection to its timeline/keyframes; show static versus animated state clearly.
- Preserve access to advanced parameters without making every common operation depend on “More.”

### Layout

```text
Left: Tools / Assets / Library | Center: composition canvas | Right: Inspector
                              | centered transport below  |
Bottom: layers                | timeline, keyframes, cache strip
```

- Left sidebar collapsible; default Tools during editing, Assets during importing, Library for reusable content. Allow pinning.
- Project/Assets is a legitimate compositing concept, also present in After Effects; hide or reorganize the panel rather than delete its functionality.
- Transport: previous/next frame, play/pause, loop, timecode, preview quality. Keep viewer zoom/overlays visually distinct.
- Proposed saved task workspaces: Design, Animate, Export.
- Timeline: zoom around playhead, jump keyframes, reveal selected properties, clearer navigation and selection.
- Useful empty states and discoverable labels for unfamiliar icons.

## Preview and rendering proposals

The blue timeline marker is the playhead. The screenshot's green strip appears to represent cached preview frames; confirm its code before changing it.

- Render the requested frame first; pointer movement and edits take priority.
- Opportunistically prefetch nearby frames, favoring the playback direction.
- Pause speculative work during active manipulation; resume after a brief idle period.
- Cancel obsolete requests after jumps and reject stale results after edits.
- Bound memory, CPU/GPU work, and concurrency; evict less useful frames. Background rendering cannot consume zero resources.
- Invalidate only affected cached results where dependency tracking makes that safe. Cache identity must account for composition state, time, quality, resolution, and relevant color settings.
- Offer reduced-quality previews independently of final export quality.
- Account for temporal/stateful effects: a requested frame may require neighboring frames or a simulation history. Do not assume every frame can be rendered independently.
- Subtle ready/rendering indicators and truthful quality states. Avoid expensive previews for every Library thumbnail simultaneously.
- Measure scrubbing latency, missed playback frames, memory usage, and foreground responsiveness before/after scheduler changes.
- Consider cache persistence and hardware acceleration after auditing what already exists.

## Other capabilities discussed

- Motion editor: approachable easing handles, spring/bounce controls, reusable motion presets.
- Reusable components: lower thirds with exposed text, colors, spacing, and animation; linked instances.
- Motion layout: padding, responsive text boxes, alignment, constraints.
- ArtCraft handoff: preserve editable text, vector paths, grouping, and asset identity; exact integration needs workflow details.
- Preview diagnostics: identify slow effects/layers, make quality and caching progress understandable.
- Export: clear presets, transparent output, batch variants, useful progress and errors. Audit existing support first.

## Architecture and upstream compatibility

- Keep document/property semantics and the shared engine command registry independent of presentation.
- Build reusable UI controls and metadata-driven effect editors, with specialized editors for effects that need them.
- Keep a generic editor so new upstream effects remain usable without a custom panel.
- UI, CLI, scripts, control channel, and MCP should continue to share commands.
- Prefer isolated additions and adapters; document project/schema changes and avoid unnecessary upstream conflicts.

## Audit underway

Checkout: `/workspace/effectcraft_`; inspected revision `e2f314f7534682f776de22874664befa99a4e6d6`, release 0.9.1, 2026-10-12. The branch's relationship to the user's published fork has not been verified.

Initial evidence: `docs/effects.md` reports 306 effects, 3,017 parameters, 280 GPU-capable effects, and 306 float/HDR-capable effects. `docs/effects-parity.md` distinguishes registration from visual fidelity: matching output against After Effects remains unmeasured. Its parameter total (3,014) differs from the generated catalog and should not be repeated as the latest count.

Verified local inventory (source/catalog audit, not a runtime or visual-equivalence test):

| Item | Count | Evidence / scope |
|---|---:|---|
| Built-in effect catalog entries | 306 | Counted unique `ec.*` rows in generated `docs/effects.md`; registry assembly in `crates/effects/src/lib.rs` |
| Effect categories | 21 | Counted generated catalog sections |
| GPU-capable catalog entries | 280 | Counted catalog GPU flags; some settings fall back to CPU per `catalog.rs` |
| Effect parameters | 3,017 total; 2,970 visible | Generated summary reports total; summed table rows count visible parameters; generator excludes ParamUi::Hidden in rows (47 hidden) |
| Toolbar tool modes | 30, grouped into 16 slots | `Tool::ALL`, `Tool::SLOTS`, `crates/ui-egui/src/state.rs`; excludes menu commands/panels |
| Bundled text animation presets | 8 | Counted `presets` entries in `crates/engine/presets/text_animators.json` |
| Bundled transition presets | 23 | Counted `presets` entries in `crates/engine/presets/transitions.json` |
| Combined bundled animation presets | 31 | The two JSON libraries; excludes user-installed presets and script extensions |

After Effects comparison:

- Adobe's public [effect list](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/effects-and-animation-presets/effect-list.html), last updated 2026-05-13, contains **290 effect rows** across 22 category tables, including Mocha AE and CINEWARE. Counted by downloading the public HTML and counting effect table rows with support flags. This is a documentation snapshot, not an exact live count of After Effects 26.5.
- This checkout's `docs/effects-parity.md` reports **298 effect names at its M9.11 count**, and **621 bundled .ffx animation presets** from its maintainer's After Effects installation. These are maintainer-reported measurements, not independently verified here against an installed Adobe app.
- That same document reports 306 `.plugin` bundles for After Effects 26.5; bundles are not individual effect names and must not be compared one-to-one with EffectCraft's 306 registry entries.
- Practical headline: After Effects has roughly 300 bundled effects. EffectCraft's catalog is already in that range, but catalog size does not establish equivalent rendered results, usability, or ecosystem support. Optional third-party effects have no universal total.
- Exact current After Effects toolbar-mode and installed-effect counts remain unverified. Do not invent a matching tool total.

Existing effects include Glow, Echo, particle systems, gradient generators, displacement, VR Digital Glitch, and many others. Existing Library features already include effect search, favorites, recently used, and GPU/depth filters. Shape gradient start/end handles are already implemented. Improve their presentation rather than treat them as new engine capabilities.

Sources to inspect: `crates/effects/src/lib.rs`, generated `docs/effects.md`, `crates/ui-egui/src/state.rs`, `crates/engine/presets/`, `docs/gaps.md`, `docs/effects-parity.md`, public Adobe documentation.

## ArtCraft / contributor implementation requirements

Read from this checkout's `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, and architecture docs:

- Pure Rust workspace; egui frontend; GPU implementations use the GPU crate's established pipeline.
- Public specifications, public behavior descriptions, and original implementations. Do not copy proprietary Adobe code, artwork, icons, fonts, presets, or plug-ins.
- Respect dependency layering and exact Tick-based timing; animatable controls use the property tree.
- Commands support automation; interactive widgets register automation IDs.
- New effects need an EffectSpec, render implementation, deterministic and behavior tests; clock-dependent effects need time-dependency registration.
- Recoverable errors, bounded input handling, no panic shortcuts or unsafe code in production paths.
- Original assets/presets need licensing and attribution entries/sidecars.
- Inspect UI changes visually; required quality gates before commits: cargo xtask ci.
- First-party ArtCraft marks have special restrictions; modified forks must remove those marks per AGENTS.md. No branding work is included in this planning task.

These requirements describe an agent-compatible development process, not a dependency on one AI model.

## Proposed sequence

1. Establish and verify the tools/effects/preset inventory; separate existence, usability, fidelity, and performance.
2. Prototype shared controls, contextual Inspector, revised sidebar, and centered playback.
3. Improve existing tools that lack direct canvas manipulation or useful feedback.
4. Measure and improve playback/scrubbing and bounded adaptive caching.
5. Add original presets and targeted effects identified by the audit, with practical demo cases.
6. Build reusable components, responsive motion layout, and the specified ArtCraft handoff.

Known crashes encountered during implementation take priority under the repository's rules. This planning task does not authorize an unrelated repair sweep.

## Next actions and unresolved details

- [x] Record the discussion in Plan.md.
- [x] Count local catalog effects, categories, parameters, tools, and preset entries.
- [x] Establish an attributable After Effects comparison and its version limitations; installed Adobe counts remain unverified.
- [x] Classify proposed additions as candidate new effects, improved existing tools, or presets/components.
- [ ] Prioritize a first tools/effects batch with concrete acceptance criteria.
- [ ] Specify ArtCraft integration and fork branding when needed.
- [x] User authorized implementation and committing/pushing to their GitHub fork on 2026-10-12.

## Discussion log

- 2026-10-12: Initial request to improve the fork, its Inspector, animation, and ArtCraft workflow.
- 2026-10-12: Expanded direction to interactive control styling, centered transport, tool-oriented left sidebar, playhead-based preview caching, and retaining upstream features beneath the new interface.
- 2026-10-12: User requested a living Plan.md and a factual tools/effects comparison before discussing original additions and ArtCraft's agent development conventions.
- 2026-10-12: Verified local catalog/tool/preset counts; counted 290 rows in Adobe's May public effect list; distinguished these from the maintainer's installation measurements. Proposed the first tools/effects batch below.

## Candidate tools and effect additions

These are recommendations for discussion, not implemented features. Names are working labels. Absence of a dedicated catalog entry is not proof that a look cannot already be built from several effects.

### Improve existing tools first

1. **Contextual tool options:** selecting Pen, Brush, Text, Roto Brush, or Puppet exposes its relevant options near the workspace. Reuse existing engine commands. One drag/edit should be one undo operation; shortcuts, saved state, and automation remain available.
2. **On-canvas effect editing:** audit and complete Mesh Warp grid dragging and Liquify brushes; extend direct manipulation for effect centers, radii, and directions. The parity docs mark Mesh Warp/Liquify interaction gaps; verify the current hooks before implementation. Existing shape gradient handles should use the same visual language.
3. **3D transforms:** audit and finish draggable move/rotate/scale handles, with axis constraints and useful multi-selection behavior. Drawing an axis indicator is not sufficient to call the tool complete.
4. **Motion behavior builder:** spring, settle, stagger, and follow controls that generate editable keyframes or explicit property behavior. Puppet follow-through and the Ease Presets script extension already exist; audit/reuse them. This is an animation tool rather than a pixel effect.
5. **Visual Library:** preview the currently focused effect/preset on representative content or the selection, show descriptions and resource hints, and apply with one undoable action. Search/favorites/recent/filtering already exist. Preview only visible/focused items with bounded work.

### Candidate original effect batch

| Candidate | Intended behavior | Relationship to existing effects |
|---|---|---|
| Animated mesh gradient | Several draggable color points with smooth interpolation and keyframed point/color motion | Extends existing fixed/ramp gradients; implement only if the multi-point control produces a distinct useful result |
| Distance-based outline | Multiple clean contour bands around an alpha silhouette, with expansion and softness | Existing stroke/glow/matte controls overlap; dedicated distance-field behavior should justify a new effect |
| Halftone print | Luminance-driven dots/lines, adjustable angle, density, and original multichannel print modes | No dedicated Halftone catalog entry found; inspect existing patterns/stylization before implementation |
| Path light ribbon | Animated luminous ribbon following an editable path, taper, falloff, and reveal timing | Stroke, Beam, Glow, and Write-on already overlap; may begin as a component/preset instead of an engine effect |
| Motion trails | Time-sampled silhouettes with width/color/fade controls and deterministic scrubbing | Echo exists; prove extra path/appearance control first; temporal dependencies make this a later, more expensive candidate |

Proposed starting engine effects: mesh gradient, distance-based outline, and halftone. Path light ribbon can begin as a reusable preset; motion trails should wait for the temporal-cache audit. None is claimed as uniquely absent from all After Effects plug-ins.

### Original preset/component batch

- Start with **40 additional original animation presets** as a proposed target, giving 71 total in the audited two libraries if all are added: 12 text, 8 shape motion, 8 backgrounds, 6 transitions, and 6 image looks.
- Previewable examples with clear names; exposed duration, intensity, direction, and color where relevant.
- Scale to composition/layer bounds where practical rather than hard-code one canvas size.
- Preserve/edit the underlying properties and keyframes; one-step undo; attribute original preset assets.
- Treat lower thirds, title cards, and logo reveals as components/templates if they create layers, rather than inflate the pixel-effect count.

### Definition of done for an effect

- Clear behavior and parameter spec, stable identifier/category, animatable property metadata.
- Original CPU renderer; GPU support when appropriate, with measured fallback behavior.
- Deterministic output for the same inputs/time/seed; behavior tests including transparent edges and HDR/non-finite inputs; safe error handling.
- Dependency/time behavior declared, including any temporal neighbors or simulation warm-up, so caching is correct.
- Undoable application/editing through commands and usable UI/automation controls.
- Original demo/preset with attribution; documented performance at specified resolution/hardware.
- Appropriate test/gate checks; fidelity comparisons where a reference behavior is being claimed. “Implemented” and “matches After Effects” stay separate claims.


## F1 implementation record — 2026-10-13

Authorized destination: `Kate-alt-69/effectcraft_`, verified fork remote and GitHub write access.
Working branch: `fork/workspace-effects`, based on `e2f314f7534682f776de22874664befa99a4e6d6` (same as origin/main at fetch).

Implemented and validated in this batch:

- Panel and workspace tabs now use selected backgrounds; shared scrub controls render as bordered numeric fields with hover/focus/drag states; inline editing, modifiers, and Tab navigation retained.
- A new Editing workspace and fresh Default layout open a Tools sidebar with all 30 tool modes and contextual shape/roto/puppet options; Project and Effects & Presets stay accessible as adjacent tabs.
- Shared centered transport beneath the canvas, including Loop; optional Preview panel retains advanced configuration.
- Inspector effect stack: bypass, edit, reorder, reset, remove, and add effect/preset via existing commands. Vector fields reserve their complete bordered width and spacing before the keyframe navigator. Existing UI fixtures explicitly open Project/Preview or arrange the tab/stack groups their interactions require; all assertions and coverage are retained.
- Three original CPU effects: Animated Mesh Gradient, Distance Outline (Euclidean distance transform), Halftone Print. Registry now 309 effects / 3,046 parameters, including 280 existing GPU-capable entries.
- Forty original presets, in separate fork JSON libraries to reduce upstream merge conflicts: 12 text, 8 shape motion, 8 backgrounds, 6 transitions, 6 image looks. Combined bundled library now 71; normalized background points adapt to each target layer.
- Restricted ArtCraft brand assets removed from the published fork; existing open EffectCraft app icon used in the header. Upstream origin credited in plain text.
- User-facing instructions and initial limitations documented in `docs/fork-workflow.md`. Five app screenshots refreshed and attributed. Editing workspace menu translations cover all ten menu languages; new Inspector, Loop, numeric tooltip, and fork About text use the six existing panel catalogs. Named font weights inherit bundled symbol fallbacks so interface bullets render consistently.
- Full validation exposed two existing GPU boundary discrepancies. HexTile now prepares bounded f64 sampling geometry before GPU sampling, matching CPU cell ties. Threshold RGB uses bounded integer quotient rounding near hard thresholds when a driver does not provide fused FMA; a regression covers every 8-bit color/alpha ratio. No fidelity tolerances were relaxed. A behavior-neutral STL parser match guard also resolves an existing Clippy warning on this toolchain. The HexTile geometry map uses eight bytes per pixel, is bounded by the device storage-buffer limit, and otherwise uses the existing CPU fallback.

Validation completed: `cargo xtask ci` passed on 2026-10-13: formatting, all-target workspace Clippy with warnings denied, native tests (3,132 passed, 26 existing ignored), dependency layering, all 31 asset attributions, and WebAssembly checks for 32 crates. The WebAssembly check retains an existing native-only `pointer_wrap::State.disabled` dead-code warning and succeeds. This includes 442 effect tests, 548 engine tests, 205 GPU tests, 227 UI unit tests, and four fork workflow pointer-event checks. All 60 UI integration targets passed; five actual app screenshots were rendered and visually inspected.

The initial full gate passed formatting and workspace Clippy, then the software Vulkan adapter retired during the existing Dust & Scratches test, causing later tests on that shared device to fail. An isolated reproduction passed with Mesa's `GALLIVM_PERF=nopt` faster shader-compilation setting. Final validation uses llvmpipe Vulkan with that setting, two renderer threads, a writable shader cache, and serial tests; no tests or application deadlines are disabled. Network access is enabled for localhost control-server tests and dependency retrieval. Local ALSA development/runtime packages supply the desktop build dependency in this managed environment. Release validation disables LTO and uses 16 codegen units to reuse builds and reduce resource cost; the repository's normal release profile remains unchanged.

Remaining plan items are future batches: Mesh Warp/Liquify canvas editing, interactive 3D transform tools, motion behavior builder, visual Library previews, adaptive caching scheduler, GPU kernels for new effects, reusable components, responsive layout, and specified ArtCraft handoff. Shape preset motions use explicit scale/rotation values and replace keys; larger canvas limits are documented. Preset duration/intensity controls are edited through resulting keyframes rather than a new application dialog.


## F2 build/source clarification — 2026-10-13

The user's Windows log clones `Kate-alt-69/effectcraft_` on `fork/workspace-effects` and compiles
EffectCraft crates from `D:\Project_Customs\Art_Craft\EffectCraft`. Downloads of FilmCraft and
craft-libs are pinned shared dependencies, not the original EffectCraft app. Their URLs and the
Cargo lockfile are retained; replacing them with this app fork would fail to resolve packages.

- Correct fork repository metadata, Help/GitHub and feedback links, startup error reporting,
  packaging support/signing links, and README clone instructions. Preserve upstream community,
  sibling-app, historical issue references, and shared-library/CI dependency sources.
- Label original upstream installers clearly; fork installer releases have not been published.
- Add root `build.ps1` for PowerShell 7: verify the existing active Rust is 1.95+, build locked in
  release mode, optionally launch the app/demo, and report native command errors. The script
  never installs, updates, or repairs Rust. Explain native Windows C++/SDK prerequisites.
- The earlier rustup install advice was unnecessary for an already compatible installation.
  The pasted rustup error reports an existing `cargo-miri.exe` link collision; the log does not
  establish Rust corruption and shows Cargo continuing compilation. User-side Rust repair is
  deferred per the user's request.
- Continue on `fork/workspace-effects`; a PR into the fork's main branch is optional.

Validation: formatting and all-target workspace Clippy passed. PowerShell 7.4 parsed the new
helper and updated signing script; six simulated-command checks on Linux passed (build, run,
demo, old Rust rejection, rustc failure, Cargo failure). Linux shell syntax and both edited XML
files parsed successfully. Native release tests were still running without reported failures
when the user explicitly instructed: “if you're waiting on Test results forget em we will check
later let me build this first!” The active gate was stopped per that instruction; full native,
layering, asset, and WebAssembly validation for F2 is deferred. No complete CI pass or native
Windows build is claimed for this batch. User instructions override the repository's usual
complete-CI-before-commit rule for this immediate build handoff.
