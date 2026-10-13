//! User-facing regression checks for the fork workspace, using real pointer events.

use effectcraft_engine::{Session, project::LayerId};
use effectcraft_ui_egui::{EffectcraftApp, state::Tool};
use egui::{Event, Pos2, Rect, pos2, vec2};
use egui_kittest::Harness;
use serde_json::json;

fn harness() -> (Harness<'static, EffectcraftApp>, u64, Vec<u64>) {
    let mut s = Session::default();
    s.execute("comp.new", json!({"name":"Fork", "width":160, "height":90, "duration":5, "frameRate":30})).unwrap();
    let layer = s.execute("layer.newSolid", json!({"name":"Plate", "color":"#608080"})).unwrap()["layer"].as_u64().unwrap();
    let effects = ["ec.stylize.halftoneprint", "ec.stylize.distanceoutline"]
        .map(|effect| s.execute("effect.apply", json!({"layers":[layer], "effect":effect})).unwrap()["effects"][0].as_u64().unwrap());
    let mut h = Harness::builder().with_size(vec2(1600.0, 1000.0)).build_eframe(move |_| EffectcraftApp::new(s));
    h.run_steps(4);
    (h, layer, effects.to_vec())
}

fn rect(h: &Harness<'_, EffectcraftApp>, id: &str) -> Rect {
    let e = h.state().auto.find(id).unwrap_or_else(|| panic!("missing {id}"));
    Rect::from_min_size(pos2(e.rect[0], e.rect[1]), vec2(e.rect[2], e.rect[3]))
}
fn click(h: &mut Harness<'_, EffectcraftApp>, at: Pos2) {
    h.event(Event::PointerMoved(at));
    h.step();
    for pressed in [true, false] {
        h.event(Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() });
        h.step();
    }
    h.run_steps(3);
}

#[test]
fn canvas_transport_steps_frames_and_shares_loop_settings() {
    let (mut h, _, _) = harness();
    let canvas = rect(&h, "viewer.area");
    let next = rect(&h, "viewer.transport.NextFrame");
    assert!(next.top() >= canvas.bottom(), "transport belongs below the canvas");
    click(&mut h, next.center());
    let comp = h.state().session.active_comp().unwrap();
    assert_eq!(h.state().session.time(), comp.frame_duration());
    let current = h.state().session.prefs.preview.current;
    let looping = h.state().session.prefs.preview.get(current).loop_;
    let at = rect(&h, "viewer.transport.Loop").center();
    click(&mut h, at);
    assert_eq!(h.state().session.prefs.preview.get(current).loop_, !looping);
}

#[test]
fn named_tools_update_toolbar_slots_and_expose_real_options() {
    let (mut h, _, _) = harness();
    let at = rect(&h, "tools.palette.Rectangle").center();
    click(&mut h, at);
    assert_eq!(h.state().ui.tool, Tool::Rectangle);
    assert_eq!(h.state().ui.slot_tools[8], Tool::Rectangle);
    let before = h.state().session.state.shape_tool.creates_mask;
    let at = rect(&h, "tools.options.createsMask").center();
    click(&mut h, at);
    assert_eq!(h.state().session.state.shape_tool.creates_mask, !before);
}

#[test]
fn rotation_fields_leave_space_for_the_sign() {
    let (h, layer, _) = harness();
    let comp = h.state().session.active_comp().unwrap();
    let rotation = comp.layer(LayerId(layer)).unwrap().props.prop("transform/rotation").unwrap();
    let base = format!("properties.prop.{}", rotation.uid);
    let revolutions = rect(&h, &format!("{base}.revolutions"));
    let degrees = rect(&h, &format!("{base}.value"));
    assert!(degrees.left() >= revolutions.right() + 4.0, "fields must not cover their separating sign");
}

#[test]
fn inspector_bypass_reorder_remove_and_undo_use_the_effect_stack() {
    let (mut h, layer, effects) = harness();
    let at = rect(&h, &format!("properties.effect.{}.enabled", effects[0])).center();
    click(&mut h, at);
    let comp = h.state().session.active_comp().unwrap();
    assert!(!comp.layer(LayerId(layer)).unwrap().props.find_group(effects[0]).unwrap().enabled);
    let at = rect(&h, &format!("properties.effect.{}.↓", effects[0])).center();
    click(&mut h, at);
    let comp = h.state().session.active_comp().unwrap();
    assert_eq!(comp.layer(LayerId(layer)).unwrap().effects().unwrap().groups().map(|g| g.uid).collect::<Vec<_>>(), vec![effects[1], effects[0]]);
    let at = rect(&h, &format!("properties.effect.{}.Remove", effects[0])).center();
    click(&mut h, at);
    assert!(h.state().session.active_comp().unwrap().layer(LayerId(layer)).unwrap().props.find_group(effects[0]).is_none());
    h.state_mut().session.execute("edit.undo", json!({})).unwrap();
    assert!(h.state().session.active_comp().unwrap().layer(LayerId(layer)).unwrap().props.find_group(effects[0]).is_some());
}
