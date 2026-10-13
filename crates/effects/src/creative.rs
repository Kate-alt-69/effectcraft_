//! Original fork effects: five-point color interpolation, Euclidean alpha contours, and
//! luminance-driven print screening. All controls use the property tree; no clock or
//! neighboring frames are needed. These effects intentionally use the CPU initially.

use crate::{Buf, EffectCtx, EffectSpec, Params, col, num, p, popup, slider};
use effectcraft_keyframe::Value;
use effectcraft_project::ParamUi;
use effectcraft_raster::{Image, Px};
use rayon::prelude::*;

const MAX_PIXELS: usize = 16_777_216;
const FAR: f64 = 1e20;
const POINTS: [(&str, &str); 5] = [("point1", "color1"), ("point2", "color2"), ("point3", "color3"), ("point4", "color4"), ("point5", "color5")];

fn finite(v: f64, fallback: f64) -> f64 {
    if v.is_finite() { v } else { fallback }
}
fn number(params: &Params, id: &str, fallback: f64, lo: f64, hi: f64) -> f64 {
    finite(params.get(id).map(Value::as_f64).unwrap_or(fallback), fallback).clamp(lo, hi)
}
fn color(params: &Params, id: &str) -> Px {
    let mut c = params.color(id).map(|v| if v.is_finite() { v.clamp(-65_504.0, 65_504.0) } else { 0.0 });
    c[3] = c[3].clamp(0.0, 1.0);
    c
}
fn premultiply(c: Px) -> Px {
    [c[0] * c[3], c[1] * c[3], c[2] * c[3], c[3]]
}
fn over(front: Px, back: Px) -> Px {
    let a = 1.0 - front[3];
    [front[0] + back[0] * a, front[1] + back[1] * a, front[2] + back[2] * a, front[3] + back[3] * a]
}
fn valid(b: &Buf) -> bool {
    (f64::MIN_POSITIVE..=1e6).contains(&b.scale)
        && b.offset.iter().all(|v| v.is_finite() && v.abs() <= 1e12)
        && (b.img.width as usize).checked_mul(b.img.height as usize).is_some_and(|n| n > 0 && n <= MAX_PIXELS && n == b.img.data.len())
}
fn spec(id: &'static str, name: &'static str, category: &'static str, params: Vec<crate::ParamSpec>, render: crate::RenderFn) -> EffectSpec {
    EffectSpec { id, name, category, params, render, gpu: false, float: true }
}
pub fn specs() -> Vec<EffectSpec> {
    let mut mesh = Vec::new();
    let defaults = [
        ([0.0, 0.0], [0.1, 0.25, 1.0, 1.0]),
        ([1.0, 0.0], [0.8, 0.1, 0.7, 1.0]),
        ([0.0, 1.0], [0.0, 0.8, 0.75, 1.0]),
        ([1.0, 1.0], [1.0, 0.45, 0.15, 1.0]),
        ([0.5, 0.5], [0.35, 0.15, 0.9, 1.0]),
    ];
    let labels = [("Point 1", "Color 1"), ("Point 2", "Color 2"), ("Point 3", "Color 3"), ("Point 4", "Color 4"), ("Point 5", "Color 5")];
    for (((point, ink), (pos, c)), (point_name, color_name)) in POINTS.into_iter().zip(defaults).zip(labels) {
        mesh.push(p(point, point_name, Value::Vec2(pos), ParamUi::Point));
        mesh.push(p(ink, color_name, col(c[0], c[1], c[2]), ParamUi::Color));
    }
    mesh.extend([
        p("falloff", "Point Influence", num(2.0), slider(0.5, 8.0, 0.5, 8.0, 2)),
        p("opacity", "Opacity", num(100.0), ParamUi::Percent),
        p("preserveAlpha", "Preserve Source Alpha", Value::Bool(false), ParamUi::Checkbox),
    ]);
    vec![
        spec("ec.generate.meshgradient", "Animated Mesh Gradient", "Generate", mesh, mesh_gradient),
        spec(
            "ec.stylize.distanceoutline",
            "Distance Outline",
            "Stylize",
            vec![
                p("width", "Band Width", num(4.0), slider(0.0, 256.0, 0.0, 64.0, 1)),
                p("offset", "Expansion", num(0.0), slider(-256.0, 256.0, -64.0, 64.0, 1)),
                p("bands", "Bands", num(1.0), slider(1.0, 8.0, 1.0, 8.0, 0)),
                p("spacing", "Band Spacing", num(4.0), slider(0.0, 256.0, 0.0, 64.0, 1)),
                p("softness", "Edge Softness", num(1.0), slider(0.0, 64.0, 0.0, 16.0, 1)),
                p("threshold", "Alpha Threshold", num(50.0), ParamUi::Percent),
                p("color", "Outline Color", col(0.3, 0.65, 1.0), ParamUi::Color),
                p("endColor", "Last Band Color", col(0.85, 0.2, 0.9), ParamUi::Color),
                p("composite", "Composite", Value::Enum(0), popup(&["Behind Source", "Outline Only", "Over Source"])),
            ],
            distance_outline,
        ),
        spec(
            "ec.stylize.halftoneprint",
            "Halftone Print",
            "Stylize",
            vec![
                p("cellSize", "Cell Size", num(12.0), slider(2.0, 256.0, 2.0, 64.0, 1)),
                p("angle", "Screen Angle", num(45.0), ParamUi::Angle),
                p("contrast", "Ink Contrast", num(1.0), slider(0.1, 4.0, 0.1, 4.0, 2)),
                p("shape", "Screen Shape", Value::Enum(0), popup(&["Dots", "Lines", "Squares"])),
                p("ink", "Ink Color", col(0.05, 0.07, 0.15), ParamUi::Color),
                p("paper", "Paper Color", col(1.0, 0.96, 0.88), ParamUi::Color),
                p("mix", "Mix with Original", num(0.0), ParamUi::Percent),
            ],
            halftone,
        ),
    ]
}
fn mesh_gradient(ctx: &EffectCtx, mut b: Buf) -> Buf {
    if !valid(&b) {
        return b;
    }
    let points: Vec<_> =
        POINTS.iter().map(|(pos, ink)| (b.to_px(ctx.params.v2(pos).map(|v| finite(v, 0.0).clamp(-1e6, 1e6))), premultiply(color(ctx.params, ink)))).collect();
    let power = number(ctx.params, "falloff", 2.0, 0.5, 8.0) * 0.5;
    let opacity = (number(ctx.params, "opacity", 100.0, 0.0, 100.0) / 100.0) as f32;
    let preserve = ctx.params.b("preserveAlpha");
    let width = b.img.width as usize;
    b.img.data.par_iter_mut().enumerate().for_each(|(i, out)| {
        let (x, y) = ((i % width) as f64 + 0.5, (i / width) as f64 + 0.5);
        let mut sum = [0.0f64; 4];
        let mut weights = 0.0;
        for ((px, py), c) in &points {
            let w = ((x - px).powi(2) + (y - py).powi(2)).max(0.25).powf(-power);
            for (v, channel) in sum.iter_mut().zip(c) {
                *v += f64::from(*channel) * w;
            }
            weights += w;
        }
        let alpha = if preserve { finite(f64::from(out[3]), 0.0).clamp(0.0, 1.0) as f32 } else { 1.0 };
        let generated = sum.map(|v| (v / weights.max(f64::MIN_POSITIVE)) as f32 * opacity * alpha);
        *out = if preserve { generated } else { over(generated, *out) };
    });
    b
}
/// Linear-time lower envelope of squared-distance parabolas. Only finite seeds enter
/// the envelope, so all-transparent rows never calculate infinity minus infinity.
fn edt_line(input: &[f64], output: &mut [f64]) {
    let mut sites: Vec<usize> = Vec::with_capacity(input.len());
    let mut starts: Vec<f64> = Vec::with_capacity(input.len());
    for (q, &cost) in input.iter().enumerate().filter(|(_, v)| **v < FAR) {
        let mut start = f64::NEG_INFINITY;
        while let Some(&v) = sites.last() {
            start = ((cost + (q as f64).powi(2)) - (input.get(v).copied().unwrap_or(FAR) + (v as f64).powi(2))) / (2.0 * (q - v) as f64);
            if start > starts.last().copied().unwrap_or(f64::NEG_INFINITY) {
                break;
            }
            sites.pop();
            starts.pop();
        }
        if sites.is_empty() {
            start = f64::NEG_INFINITY;
        }
        sites.push(q);
        starts.push(start);
    }
    let mut k = 0usize;
    for (q, value) in output.iter_mut().enumerate() {
        while starts.get(k.saturating_add(1)).is_some_and(|s| *s <= q as f64) {
            k = k.saturating_add(1);
        }
        *value = sites.get(k).map_or(FAR, |&v| input.get(v).copied().unwrap_or(FAR) + (q as f64 - v as f64).powi(2));
    }
}
fn distances(img: &Image, threshold: f32, inside: bool) -> Vec<f64> {
    let (w, h) = (img.width as usize, img.height as usize);
    let mut result: Vec<_> = img.data.iter().map(|p| if (p[3].is_finite() && p[3] > threshold) == inside { 0.0 } else { FAR }).collect();
    let mut input = vec![0.0; w.max(h)];
    let mut output = vec![0.0; w.max(h)];
    for row in result.chunks_mut(w) {
        edt_line(row, output.get_mut(..w).unwrap_or_default());
        if let Some(values) = output.get(..w) {
            row.copy_from_slice(values);
        }
    }
    for x in 0..w {
        for (y, slot) in input.iter_mut().take(h).enumerate() {
            *slot = result.get(y.saturating_mul(w).saturating_add(x)).copied().unwrap_or(FAR);
        }
        edt_line(input.get(..h).unwrap_or_default(), output.get_mut(..h).unwrap_or_default());
        for (y, &d) in output.iter().take(h).enumerate() {
            if let Some(value) = result.get_mut(y.saturating_mul(w).saturating_add(x)) {
                *value = d;
            }
        }
    }
    result
}
fn distance_outline(ctx: &EffectCtx, mut b: Buf) -> Buf {
    if !valid(&b) {
        return b;
    }
    let width = number(ctx.params, "width", 4.0, 0.0, 256.0) * b.scale;
    if width == 0.0 {
        return b;
    }
    let offset = number(ctx.params, "offset", 0.0, -256.0, 256.0) * b.scale;
    let bands = number(ctx.params, "bands", 1.0, 1.0, 8.0).round() as usize;
    let spacing = number(ctx.params, "spacing", 4.0, 0.0, 256.0) * b.scale;
    let soft = (number(ctx.params, "softness", 1.0, 0.0, 64.0) * b.scale).max(0.5);
    let threshold = (number(ctx.params, "threshold", 50.0, 0.0, 100.0) / 100.0) as f32;
    let padding = (offset.max(0.0) + bands as f64 * width + bands.saturating_sub(1) as f64 * spacing + soft + 1.0).ceil();
    if !padding.is_finite() || padding > 4096.0 {
        return b;
    }
    let pad = padding as u32;
    let dims = b.img.width.checked_add(pad.saturating_mul(2)).zip(b.img.height.checked_add(pad.saturating_mul(2)));
    if dims.is_none_or(|(w, h)| (w as usize).checked_mul(h as usize).is_none_or(|n| n > MAX_PIXELS)) {
        return b;
    }
    b.pad(pad);
    let outside = distances(&b.img, threshold, true);
    let inside = distances(&b.img, threshold, false);
    let (a, z) = (color(ctx.params, "color"), color(ctx.params, "endColor"));
    let composite = ctx.params.e("composite");
    b.img.data.par_iter_mut().enumerate().for_each(|(i, source)| {
        let d = if source[3].is_finite() && source[3] > threshold {
            0.5 - inside.get(i).copied().unwrap_or(FAR).sqrt()
        } else {
            outside.get(i).copied().unwrap_or(FAR).sqrt() - 0.5
        };
        let mut coverage = 0.0f64;
        let mut ink = a;
        for band in 0..bands {
            let start = offset + band as f64 * (width + spacing);
            let c = ((d - start + soft * 0.5) / soft).clamp(0.0, 1.0) * ((start + width - d + soft * 0.5) / soft).clamp(0.0, 1.0);
            if c > coverage {
                coverage = c;
                let t = band as f32 / bands.saturating_sub(1).max(1) as f32;
                ink = std::array::from_fn(|i| a[i] + (z[i] - a[i]) * t);
            }
        }
        let contour = premultiply([ink[0], ink[1], ink[2], ink[3] * coverage as f32]);
        *source = match composite {
            1 => contour,
            2 => over(contour, *source),
            _ => over(*source, contour),
        };
    });
    b
}
fn halftone(ctx: &EffectCtx, mut b: Buf) -> Buf {
    if !valid(&b) {
        return b;
    }
    let size = (number(ctx.params, "cellSize", 12.0, 2.0, 256.0) * b.scale).max(1.0);
    let (sin, cos) = number(ctx.params, "angle", 45.0, -1e6, 1e6).to_radians().sin_cos();
    let contrast = number(ctx.params, "contrast", 1.0, 0.1, 4.0);
    let mix = (number(ctx.params, "mix", 0.0, 0.0, 100.0) / 100.0) as f32;
    let (ink, paper) = (color(ctx.params, "ink"), color(ctx.params, "paper"));
    let shape = ctx.params.e("shape");
    let source = b.img.clone();
    let w = source.width as usize;
    let origin = b.offset;
    b.img.data.par_iter_mut().enumerate().for_each(|(i, out)| {
        let (x, y) = ((i % w) as f64 + 0.5 - origin[0], (i / w) as f64 + 0.5 - origin[1]);
        let (u, v) = (x * cos + y * sin, -x * sin + y * cos);
        let (cu, cv) = ((u / size).floor() * size + size * 0.5, (v / size).floor() * size + size * 0.5);
        let sample = source.sample_bilinear_clamped(cu * cos - cv * sin + origin[0] - 0.5, cu * sin + cv * cos + origin[1] - 0.5);
        let alpha = finite(f64::from(sample[3]), 0.0).clamp(0.0, 1.0);
        let luma = if alpha > 1e-8 { (0.2126 * f64::from(sample[0]) + 0.7152 * f64::from(sample[1]) + 0.0722 * f64::from(sample[2])) / alpha } else { 1.0 };
        let amount = (1.0 - finite(luma, 1.0)).clamp(0.0, 1.0).powf(contrast);
        let (du, dv) = ((u - cu).abs(), (v - cv).abs());
        let (radius, distance) = match shape {
            1 => (amount * size * 0.5, du),
            2 => (amount.sqrt() * size * 0.5, du.max(dv)),
            _ => ((amount / std::f64::consts::PI).sqrt() * size, du.hypot(dv)),
        };
        let coverage = if amount <= 0.0 {
            0.0
        } else if amount >= 1.0 {
            1.0
        } else {
            (radius - distance + 0.5).clamp(0.0, 1.0) as f32
        };
        let original = *out;
        let a = finite(f64::from(original[3]), 0.0).clamp(0.0, 1.0) as f32;
        let screened = premultiply(std::array::from_fn(|c| paper[c] + (ink[c] - paper[c]) * coverage)).map(|v| v * a);
        *out = std::array::from_fn(|c| screened[c] * (1.0 - mix) + original[c] * mix);
    });
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    fn defaults(id: &str, size: [f64; 2]) -> Params {
        Params { values: crate::find(id).unwrap().params.iter().map(|p| (p.id.to_string(), crate::default_value(p, size))).collect() }
    }
    fn render(id: &str, params: &Params, image: Image) -> Buf {
        let ctx =
            EffectCtx { params, time: 0.0, layer_size: [f64::from(image.width), f64::from(image.height)], seed: 1, adjustment: false, env: Default::default() };
        (crate::find(id).unwrap().render)(&ctx, Buf { img: image, offset: [0.0; 2], scale: 1.0 })
    }
    #[test]
    fn mesh_points_animate_keep_hdr_and_respect_alpha() {
        let id = "ec.generate.meshgradient";
        let mut p = defaults(id, [9.0, 9.0]);
        for (_, ink) in POINTS {
            p.values.insert(ink.into(), Value::Color([4.0, 0.5, 0.25, 1.0]));
        }
        assert!(render(id, &p, Image::new(9, 9)).img.data.iter().all(|v| (v[0] - 4.0).abs() < 1e-5 && v[3] == 1.0));
        p.values.insert("preserveAlpha".into(), Value::Bool(true));
        assert!(render(id, &p, Image::new(9, 9)).img.data.iter().all(|v| *v == [0.0; 4]));
        p = defaults(id, [9.0, 9.0]);
        let before = render(id, &p, Image::new(9, 9));
        p.values.insert("point5".into(), Value::Vec2([8.5, 1.5]));
        assert_ne!(before.img.data, render(id, &p, Image::new(9, 9)).img.data);
    }
    #[test]
    fn euclidean_transform_handles_empty_and_diagonal_sites() {
        let mut img = Image::new(5, 5);
        assert!(distances(&img, 0.5, true).iter().all(|v| *v == FAR));
        img.data[12] = [1.0; 4];
        let d = distances(&img, 0.5, true);
        assert_eq!((d[12], d[0], d[1]), (0.0, 8.0, 5.0));
        let mut out = [0.0; 5];
        edt_line(&[FAR, 0.0, FAR, FAR, 0.0], &mut out);
        assert_eq!(out, [1.0, 0.0, 1.0, 1.0, 0.0]);
    }
    #[test]
    fn outline_grows_bounds_without_inventing_empty_contours() {
        let id = "ec.stylize.distanceoutline";
        let p = defaults(id, [9.0, 9.0]);
        assert!(render(id, &p, Image::new(9, 9)).img.data.iter().all(|v| v[3] == 0.0));
        let mut image = Image::new(9, 9);
        image.data[40] = [1.0; 4];
        let out = render(id, &p, image);
        assert!(out.img.width > 9 && out.offset[0] > 0.0);
        let (cx, cy) = out.to_px([4.0, 4.0]);
        assert_eq!(out.img.get(cx as i64, cy as i64), [1.0; 4]);
        assert!(out.img.get(cx as i64 + 2, cy as i64)[3] > 0.9);
        assert_eq!(out.img.get(cx as i64 + 6, cy as i64)[3], 0.0);
    }
    #[test]
    fn halftone_tracks_luminance_preserves_alpha_and_mix_is_identity() {
        let id = "ec.stylize.halftoneprint";
        let mut p = defaults(id, [16.0, 16.0]);
        assert!(render(id, &p, Image::filled(16, 16, [0.0, 0.0, 0.0, 1.0])).img.data.iter().all(|v| v[0] < 0.1));
        assert!(render(id, &p, Image::filled(16, 16, [1.0; 4])).img.data.iter().all(|v| v[0] == 1.0));
        assert!(render(id, &p, Image::new(16, 16)).img.data.iter().all(|v| *v == [0.0; 4]));
        p.values.insert("mix".into(), num(100.0));
        let original = Image::filled(16, 16, [2.0, 0.5, 0.2, 0.5]);
        assert_eq!(render(id, &p, original.clone()).img.data, original.data);
    }
    #[test]
    fn effects_guard_bad_geometry_and_parameters_and_are_deterministic() {
        for spec in specs() {
            let mut p = defaults(spec.id, [12.0, 8.0]);
            let image = Image::filled(12, 8, [0.2, 0.1, 0.05, 0.5]);
            assert_eq!(render(spec.id, &p, image.clone()).img.data, render(spec.id, &p, image.clone()).img.data);
            for value in p.values.values_mut() {
                match value {
                    Value::Scalar(v) => *v = f64::NAN,
                    Value::Vec2(v) => *v = [f64::INFINITY; 2],
                    Value::Color(v) => *v = [f64::NAN; 4],
                    _ => {}
                }
            }
            assert!(render(spec.id, &p, image).img.data.iter().flatten().all(|v| v.is_finite()));
            assert!(render(spec.id, &p, Image::new(0, 0)).img.data.is_empty());
            let mut malformed = Image::new(2, 2);
            malformed.data.pop();
            assert_eq!(render(spec.id, &p, malformed).img.data.len(), 3);
            let ctx = EffectCtx { params: &p, time: 0.0, layer_size: [12.0, 8.0], seed: 1, adjustment: false, env: Default::default() };
            for (scale, offset) in [(0.0, [0.0; 2]), (f64::NAN, [0.0; 2]), (f64::MAX, [0.0; 2]), (1.0, [f64::MAX; 2])] {
                let image = Image::filled(12, 8, [0.2, 0.1, 0.05, 0.5]);
                let original = image.data.clone();
                assert_eq!((spec.render)(&ctx, Buf { img: image, offset, scale }).img.data, original, "invalid geometry passes through without overflow");
            }
        }
    }
}
