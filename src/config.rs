use super::*;

#[derive(Deserialize)]
pub struct Config {
    pub sky_color: Rgba<f32>,
    pub gravity: f32,
    pub throw_speed: f32,
    pub throw_angle: f32,
    pub item_scale: f32,
    pub item_hold_scale: f32,
    pub hand_radius: f32,
    pub item_max_w: f32,
    pub throw_target_height: f32,
    pub ui_fov: f32,
    pub fov: f32,
    pub earth_radius: f32,
    pub ride_speed: f32,
    pub camera_height: f32,
    pub camera_rot: f32,
    pub road_width: f32,
    pub mailbox_size: f32,
    pub distance_between_mailboxes: f32,
    pub throw_time: f32,
    pub item_throw_max_w: f32,
    pub item_throw_scale: f32,
    pub throw_height: f32,
    pub mailbox_colors: Vec<Rgba<f32>>,
    pub double_mailbox_probability: f64,
    pub time_scale: f32,
    pub start_time: f32,
    pub lives: usize,
    pub juggling_score_multiplier: f32,
    pub deliver_score: f32,
    pub spawn_distance: f32,
    pub despawn_distance: f32,
    pub hand_rotation: f32,
    pub throw_animation_time: f32,
    pub throw_hand_distance: f32,
    pub error_animation_time: f32,
    pub error_animation_distance: f32,
    pub error_animation_freq: f32,
    pub particle_count: usize,
    pub particle_speed: f32,
    pub particle_size: f32,
    pub particle_lifetime: f32,
    pub explosion_color: Rgba<f32>,
}
