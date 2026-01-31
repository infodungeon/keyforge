use keyforge_model::geometry::KeyboardGeometry;
use std::error::Error;
use std::fmt::Write;

/// Generates an SVG visualization of the keyboard physics (heatmaps, distances).
///
/// # Errors
/// Returns an error if writing to the output string fails.
pub fn generate_physics_svg(
    geo: &KeyboardGeometry,
    heatmap: &[f32],
    _penalties: &[f32],
) -> Result<String, Box<dyn Error>> {
    let mut svg = String::new();

    // SVG Header
    writeln!(
        &mut svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 400\">"
    )?;

    // Styles
    writeln!(
        &mut svg,
        "<style>.key {{ stroke: #333; stroke-width: 1; }} .home {{ stroke: #3b82f6; stroke-width: 2; }}</style>"
    )?;

    for key in geo.keys() {
        let x = key.x.to_f32() * 50.0 + 50.0;
        let y = key.y.to_f32() * 50.0 + 50.0;
        let w = key.w * 45.0;
        let h = key.h * 45.0;

        let freq = heatmap.get(key.index).copied().unwrap_or(0.0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let intensity = (freq * 255.0).min(255.0) as u8;
        let color = format!("rgb({}, {}, {})", 255 - intensity, 255 - intensity, 255);

        let class = if key.is_home { "key home" } else { "key" };

        writeln!(
            &mut svg,
            "<rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" fill=\"{color}\" class=\"{class}\" rx=\"4\" />"
        )?;

        // Label
        writeln!(
            &mut svg,
            "<text x=\"{}\" y=\"{}\" font-size=\"10\" text-anchor=\"middle\" fill=\"#000\">{}</text>",
            x + w / 2.0,
            y + h / 2.0 + 4.0,
            key.label
        )?;
    }

    writeln!(&mut svg, "</svg>")?;

    Ok(svg)
}
