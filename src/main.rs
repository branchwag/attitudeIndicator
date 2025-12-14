use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 600.0])
            .with_title("Aircraft Attitude Indicator"),
        ..Default::default()
    };

    eframe::run_native(
        "attitude-indicator",
        options,
        Box::new(|_cc| Ok(Box::new(AttitudeIndicatorApp::default()))),
    )
}

struct AttitudeIndicatorApp {
    pitch: f32,  // -90 to +90 degrees
    roll: f32,   // -180 to +180 degrees
}

impl Default for AttitudeIndicatorApp {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            roll: 0.0,
        }
    }
}

impl eframe::App for AttitudeIndicatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // Main attitude indicator display
                let available_size = ui.available_size();
                let indicator_size = available_size.y - 150.0;
                let (response, painter) = ui.allocate_painter(
                    Vec2::new(available_size.x, indicator_size),
                    egui::Sense::hover(),
                );

                let rect = response.rect;
                let center = rect.center();
                let radius = (indicator_size.min(available_size.x) / 2.0) - 20.0;

                // Draw the attitude indicator
                draw_attitude_indicator(&painter, center, radius, self.pitch, self.roll);

                ui.add_space(10.0);

                // Control sliders
                ui.horizontal(|ui| {
                    ui.label("Pitch:");
                    ui.add(
                        egui::Slider::new(&mut self.pitch, -90.0..=90.0)
                            .suffix("°")
                            .step_by(1.0),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Roll:");
                    ui.add(
                        egui::Slider::new(&mut self.roll, -180.0..=180.0)
                            .suffix("°")
                            .step_by(1.0),
                    );
                });

                ui.add_space(5.0);

                // Reset button
                if ui.button("Reset to Level Flight").clicked() {
                    self.pitch = 0.0;
                    self.roll = 0.0;
                }
            });
        });
    }
}

fn draw_attitude_indicator(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    pitch: f32,
    roll: f32,
) {
    // Colors
    let sky_color = Color32::from_rgb(0, 153, 255);
    let ground_color = Color32::from_rgb(153, 102, 51);
    let markings_color = Color32::WHITE;
    let plane_symbol_color = Color32::YELLOW;
    let bezel_color = Color32::DARK_GRAY;

    // Create circular clipping region
    let clip_rect = Rect::from_center_size(center, Vec2::splat(radius * 2.0));
    painter.with_clip_rect(clip_rect).add(egui::Shape::Noop); // Use with_clip_rect instead

    // Save the current state and apply rotation for roll
    let roll_rad = roll.to_radians();
    
    // Calculate pitch offset (4 pixels per degree)
    let pitch_offset = pitch * 4.0;

    // We need to draw the rotated horizon
    // First, create a very large rectangle for sky and ground
    let large_size = radius * 4.0;
    
    // Draw sky (upper half) with clipping
    let sky_shape = draw_rotated_rect_shape(
        center,
        large_size,
        large_size / 2.0,
        Pos2::new(0.0, -large_size / 4.0 - pitch_offset),
        roll_rad,
        sky_color,
    );
    painter.with_clip_rect(clip_rect).add(sky_shape);

    // Draw ground (lower half) with clipping
    let ground_shape = draw_rotated_rect_shape(
        center,
        large_size,
        large_size / 2.0,
        Pos2::new(0.0, large_size / 4.0 - pitch_offset),
        roll_rad,
        ground_color,
    );
    painter.with_clip_rect(clip_rect).add(ground_shape);

    // Draw horizon line
    let horizon_start = rotate_point(
        Pos2::new(-radius, -pitch_offset),
        Pos2::ZERO,
        roll_rad,
    );
    let horizon_end = rotate_point(
        Pos2::new(radius, -pitch_offset),
        Pos2::ZERO,
        roll_rad,
    );
    painter.with_clip_rect(clip_rect).line_segment(
        [center + horizon_start.to_vec2(), center + horizon_end.to_vec2()],
        Stroke::new(3.0, markings_color),
    );

    // Draw pitch ladder
    for i in -9..=9i32 {
        if i == 0 {
            continue; // Skip horizon line
        }

        let y = -pitch_offset - (i as f32 * 40.0); // 40 pixels = 10 degrees
        let line_length = if i % 3 == 0 { 60.0 } else { 30.0 };

        let line_start = rotate_point(
            Pos2::new(-line_length, y),
            Pos2::ZERO,
            roll_rad,
        );
        let line_end = rotate_point(
            Pos2::new(line_length, y),
            Pos2::ZERO,
            roll_rad,
        );

        painter.with_clip_rect(clip_rect).line_segment(
            [center + line_start.to_vec2(), center + line_end.to_vec2()],
            Stroke::new(2.0, markings_color),
        );

        // Draw pitch values
        let pitch_value = (i * 10).abs();
        let text = format!("{}°", pitch_value);
        
        let text_pos_right = rotate_point(
            Pos2::new(line_length + 5.0, y),
            Pos2::ZERO,
            roll_rad,
        );
        let text_pos_left = rotate_point(
            Pos2::new(-line_length - 25.0, y),
            Pos2::ZERO,
            roll_rad,
        );

        painter.with_clip_rect(clip_rect).text(
            center + text_pos_right.to_vec2(),
            egui::Align2::LEFT_CENTER,
            &text,
            egui::FontId::proportional(12.0),
            markings_color,
        );
        painter.with_clip_rect(clip_rect).text(
            center + text_pos_left.to_vec2(),
            egui::Align2::LEFT_CENTER,
            &text,
            egui::FontId::proportional(12.0),
            markings_color,
        );
    }

    // Draw outer bezel circle (no clipping)
    painter.circle_stroke(center, radius, Stroke::new(4.0, bezel_color));

    // Draw roll indicator triangle at top
    let triangle_size = 10.0;
    let tri_top = center + Vec2::new(0.0, -radius + 5.0);
    let tri_left = tri_top + Vec2::new(-triangle_size, triangle_size);
    let tri_right = tri_top + Vec2::new(triangle_size, triangle_size);
    painter.add(egui::Shape::convex_polygon(
        vec![tri_top, tri_left, tri_right],
        Color32::WHITE,
        Stroke::NONE,
    ));

    // Draw roll scale markers
    for angle in (0..360).step_by(30) {
        let angle_rad = (angle as f32 - 90.0).to_radians(); // -90 to start from top
        let marker_length = if angle % 90 == 0 { 15.0 } else { 10.0 };
        let stroke_width = if angle == 0 { 3.0 } else { 2.0 };

        let outer_pos = center + Vec2::new(
            (radius - 5.0) * angle_rad.cos(),
            (radius - 5.0) * angle_rad.sin(),
        );
        let inner_pos = center + Vec2::new(
            (radius - marker_length) * angle_rad.cos(),
            (radius - marker_length) * angle_rad.sin(),
        );

        painter.line_segment(
            [outer_pos, inner_pos],
            Stroke::new(stroke_width, Color32::WHITE),
        );

        // Draw angle labels
        if angle > 0 && angle < 180 {
            let label_pos = center + Vec2::new(
                (radius - marker_length - 20.0) * angle_rad.cos(),
                (radius - marker_length - 20.0) * angle_rad.sin(),
            );
            painter.text(
                label_pos,
                egui::Align2::CENTER_CENTER,
                angle.to_string(),
                egui::FontId::proportional(11.0),
                Color32::WHITE,
            );
        } else if angle > 180 {
            let label_pos = center + Vec2::new(
                (radius - marker_length - 20.0) * angle_rad.cos(),
                (radius - marker_length - 20.0) * angle_rad.sin(),
            );
            painter.text(
                label_pos,
                egui::Align2::CENTER_CENTER,
                (angle as i32 - 360).to_string(),
                egui::FontId::proportional(11.0),
                Color32::WHITE,
            );
        }
    }

    // Draw fixed aircraft symbol (not rotated)
    // Horizontal wings
    painter.line_segment(
        [center + Vec2::new(-60.0, 0.0), center + Vec2::new(60.0, 0.0)],
        Stroke::new(3.0, plane_symbol_color),
    );

    // Center dot
    painter.circle_filled(center, 5.0, plane_symbol_color);

    // Vertical center line
    painter.line_segment(
        [center + Vec2::new(0.0, -10.0), center + Vec2::new(0.0, 10.0)],
        Stroke::new(3.0, plane_symbol_color),
    );
}

fn rotate_point(point: Pos2, origin: Pos2, angle: f32) -> Pos2 {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let dx = point.x - origin.x;
    let dy = point.y - origin.y;
    
    Pos2::new(
        origin.x + dx * cos_a - dy * sin_a,
        origin.y + dx * sin_a + dy * cos_a,
    )
}

fn draw_rotated_rect_shape(
    center: Pos2,
    width: f32,
    height: f32,
    offset: Pos2,
    angle: f32,
    color: Color32,
) -> egui::Shape {
    let half_w = width / 2.0;
    let half_h = height / 2.0;

    let corners = [
        Pos2::new(offset.x - half_w, offset.y - half_h),
        Pos2::new(offset.x + half_w, offset.y - half_h),
        Pos2::new(offset.x + half_w, offset.y + half_h),
        Pos2::new(offset.x - half_w, offset.y + half_h),
    ];

    let rotated_corners: Vec<Pos2> = corners
        .iter()
        .map(|&corner| {
            let rotated = rotate_point(corner, Pos2::ZERO, angle);
            center + rotated.to_vec2()
        })
        .collect();

    egui::Shape::convex_polygon(
        rotated_corners,
        color,
        Stroke::NONE,
    )
}
