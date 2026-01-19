// libs/keyforge-export/src/viz/physics.rs

use keyforge_model::geometry::KeyboardGeometry;
use std::fmt::Write;

/// Generates a basic SVG visualization of the keyboard geometry.
#[must_use]
pub fn generate_physics_svg(geo: &KeyboardGeometry) -> String {
    let mut svg = String::new();

    // Determine bounds
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for key in &geo.keys {
        if key.x < min_x {
            min_x = key.x;
        }
        if key.y < min_y {
            min_y = key.y;
        }
        if key.x + key.w > max_x {
            max_x = key.x + key.w;
        }
        if key.y + key.h > max_y {
            max_y = key.y + key.h;
        }
    }

    let padding = 10.0;
    let width = max_x - min_x + padding * 2.0;
    let height = max_y - min_y + padding * 2.0;

    let _ = write!(
        svg,
        r#"<svg viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg">"#
    );
    svg.push_str(r##"<rect width="100%" height="100%" fill="#f8f9fa" />"##);

    for key in &geo.keys {
        let x = key.x - min_x + padding;
        let y = key.y - min_y + padding;
        let w = key.w;
        let h = key.h;

        let color = if key.row.0 == geo.home_row {
            "#e9ecef"
        } else {
            "#ffffff"
        };

        let _ = write!(
            svg,
            r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="2" fill="{color}" stroke="#dee2e6" stroke-width="0.5" />"##
        );

        // Add label if present
        if !key.label.is_empty() {
            let _ = write!(
                svg,
                r##"<text x="{}" y="{}" font-family="sans-serif" font-size="3" fill="#495057" text-anchor="middle" alignment-baseline="middle">{}</text>"##,
                x + w / 2.0,
                y + h / 2.0,
                key.label
            );
        }
    }

    svg.push_str("</svg>");
    svg
}
