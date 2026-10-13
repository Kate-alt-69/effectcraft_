//! Discoverable tool palette with context-sensitive options. Tool modes remain frontend
//! state; drawing and option changes still use the existing engine commands.

use crate::dock::PanelKind;
use crate::state::Tool;
use crate::{EffectcraftApp, widgets};
use egui::{Rect, vec2};
use serde_json::json;

fn execute(app: &mut EffectcraftApp, command: &str, params: serde_json::Value) {
    if let Err(e) = app.session.execute(command, params) {
        app.ui.status = e.to_string();
    }
}

pub fn show(app: &mut EffectcraftApp, ui: &mut egui::Ui, rect: Rect) {
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect.shrink(10.0)), |ui| {
        ui.set_clip_rect(rect.intersect(ui.clip_rect()));
        egui::ScrollArea::vertical().id_salt("tool-palette").show(ui, |ui| {
            let tool = app.ui.tool;
            ui.strong(tool.label());
            if let Some(shortcut) = tool.shortcut() {
                ui.label(egui::RichText::new(format!("Shortcut: {shortcut}")).small().color(app.tokens.text_dim));
            }
            ui.add_space(6.0);
            if tool.is_shape() || matches!(tool, Tool::Pen | Tool::PenAdd | Tool::PenDelete | Tool::PenConvert | Tool::MaskFeather) {
                let mut mask = app.session.state.shape_tool.creates_mask;
                let r = ui.checkbox(&mut mask, "Create mask on selected shape");
                app.auto.add_response("tools.options.createsMask", &r, "Create mask");
                if r.changed() {
                    execute(app, "shape.toolOptions", json!({"createsMask": mask}));
                }
                let mut width = app.session.state.shape_tool.stroke_width;
                ui.horizontal(|ui| {
                    ui.label("Stroke");
                    let r = ui.add(widgets::drag_value(egui::DragValue::new(&mut width).range(0.0..=1000.0).suffix(" px")));
                    app.auto.add_response("tools.options.strokeWidth", &r, "Stroke Width");
                    if r.changed() {
                        execute(app, "shape.toolOptions", json!({"strokeWidth": width}));
                    }
                });
                ui.label(egui::RichText::new("Fill and stroke colors are in the top toolbar.").small());
            }
            if tool.roto_kind(false).is_some() {
                let refine = tool.roto_kind(false).is_some_and(|k| k.starts_with("refine"));
                let (key, mut diameter) =
                    if refine { ("refineDiameter", app.session.state.roto.refine_diameter) } else { ("diameter", app.session.state.roto.diameter) };
                ui.horizontal(|ui| {
                    ui.label("Diameter");
                    let r = ui.add(widgets::drag_value(egui::DragValue::new(&mut diameter).range(1.0..=5000.0).suffix(" px")));
                    app.auto.add_response("tools.options.rotoDiameter", &r, "Roto diameter");
                    if r.changed() {
                        execute(app, "roto.options", json!({key: diameter}));
                    }
                });
            }
            if tool.puppet_kind().is_some() {
                let mut show = app.session.state.puppet.show_mesh;
                let r = ui.checkbox(&mut show, "Show deformation mesh");
                app.auto.add_response("tools.options.showMesh", &r, "Show mesh");
                if r.changed() {
                    execute(app, "puppet.mesh", json!({"showMesh": show}));
                }
                ui.label(egui::RichText::new("Ctrl/Cmd-drag a pin to record motion.").small());
            }
            let options = match tool {
                Tool::Type | Tool::TypeVertical => Some((PanelKind::Character, "Typography options")),
                Tool::Brush | Tool::Clone | Tool::Eraser => Some((PanelKind::Paint, "Paint options")),
                Tool::RotoBrush | Tool::RefineEdge => Some((PanelKind::EffectControls, "Roto controls")),
                _ if tool.puppet_kind().is_some() => Some((PanelKind::EffectControls, "Puppet controls")),
                _ => None,
            };
            if let Some((panel, label)) = options {
                let r = ui.button(label);
                app.auto.add_response("tools.options.open", &r, label);
                if r.clicked() {
                    app.show_panel(panel);
                }
            }
            ui.separator();
            for (group, tools) in [
                ("Navigate & transform", Tool::ALL.get(..8).unwrap_or_default()),
                ("Shapes & paths", Tool::ALL.get(8..18).unwrap_or_default()),
                ("Text & paint", Tool::ALL.get(18..23).unwrap_or_default()),
                ("Roto & puppet", Tool::ALL.get(23..).unwrap_or_default()),
            ] {
                ui.strong(group);
                for &candidate in tools {
                    let label = candidate.label().trim_end_matches(" Tool");
                    let r = ui.add_sized(vec2(ui.available_width().max(40.0), 25.0), egui::Button::new(label).selected(app.ui.tool == candidate));
                    app.auto.add_response(&format!("tools.palette.{candidate:?}"), &r, candidate.label());
                    let r = if let Some(shortcut) = candidate.shortcut() { r.on_hover_text(format!("{} · {shortcut}", candidate.label())) } else { r };
                    if r.clicked() {
                        app.ui.tool = candidate;
                        if let Some(index) = Tool::SLOTS.iter().position(|slot| slot.contains(&candidate))
                            && let Some(saved) = app.ui.slot_tools.get_mut(index)
                        {
                            *saved = candidate;
                        }
                    }
                }
                ui.add_space(8.0);
            }
            ui.separator();
            for (panel, label) in [(PanelKind::Project, "Browse project assets"), (PanelKind::EffectsPresets, "Browse effects & presets")] {
                let r = ui.button(label);
                app.auto.add_response(&format!("tools.open.{}", panel.id()), &r, label);
                if r.clicked() {
                    app.show_panel(panel);
                }
            }
        });
    });
}
