use bevy::prelude::*;

/// Performs a standard 2-joint 2D IK calculation, returning the intermediate elbow/knee coordinate.
pub fn solve_ik_2d(start: Vec2, target: Vec2, l1: f32, l2: f32, flip: bool) -> Vec2 {
    let to_target = target - start;
    let dist = to_target.length();
    
    // Avoid numerical collapse when targets exceed standard bounds
    let max_reach = l1 + l2;
    let clamped_dist = dist.clamp(0.01, max_reach - 0.1);
    let clamped_target = if dist > 0.0 {
        start + to_target.normalize() * clamped_dist
    } else {
        start + Vec2::new(0.0, -clamped_dist)
    };
    
    let to_clamped = clamped_target - start;
    let d = to_clamped.length();
    
    let cos_a = (l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d);
    let angle_a = cos_a.clamp(-1.0, 1.0).acos();
    
    let base_angle = to_clamped.y.atan2(to_clamped.x);
    let angle = if flip {
        base_angle + angle_a
    } else {
        base_angle - angle_a
    };
    
    start + Vec2::new(angle.cos(), angle.sin()) * l1
}

/// Renders a thick, satisfying 3D-esque noodle segment by sweeping connected Bezier lines.
pub fn draw_connected_noodle(
    gizmos: &mut Gizmos,
    start: Vec2,
    joint: Vec2,
    target: Vec2,
    color: Color,
    scale: f32,
) {
    let segments = 6;
    for i in 0..segments {
        let t1 = i as f32 / segments as f32;
        let t2 = (i + 1) as f32 / segments as f32;
        
        let p1 = (1.0 - t1).powi(2) * start + 2.0 * (1.0 - t1) * t1 * joint + t1.powi(2) * target;
        let p2 = (1.0 - t2).powi(2) * start + 2.0 * (1.0 - t2) * t2 * joint + t2.powi(2) * target;
        
        // Draw 3 closely-spaced lines parallel to achieve thickness!
        gizmos.line_2d(p1, p2, color);
        gizmos.line_2d(p1 + Vec2::new(1.0, 0.0) * scale, p2 + Vec2::new(1.0, 0.0) * scale, color);
        gizmos.line_2d(p1 + Vec2::new(-1.0, 0.0) * scale, p2 + Vec2::new(-1.0, 0.0) * scale, color);
    }
}
