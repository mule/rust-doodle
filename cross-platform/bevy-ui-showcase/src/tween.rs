use bevy::math::curve::EaseFunction;
use bevy::prelude::*;

/// Generic tween component. Each concrete `T` is its own ECS-distinct
/// component type — `Tween<f32>` and `Tween<Color>` are unrelated.
/// Three concrete advance systems below cover the property types this
/// crate animates: `f32` for scale, `Color` for crossfades, `Val` for
/// slide positions.
#[derive(Component)]
pub struct Tween<T: Clone + Send + Sync + 'static> {
    pub start: T,
    pub end: T,
    pub elapsed: f32,
    pub duration: f32,
    pub easing: EaseFunction,
}

/// Advance every `Tween<f32>` and write the interpolated value into the
/// entity's `UiTransform.scale` (uniform x and y). Removes the tween
/// component when complete.
pub fn advance_f32_tweens(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Tween<f32>, &mut UiTransform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tween, mut transform) in &mut q {
        tween.elapsed += dt;
        let raw = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
        let eased = tween.easing.sample(raw).unwrap_or(raw);
        let value = tween.start.lerp(tween.end, eased);
        transform.scale = Vec2::splat(value);
        if raw >= 1.0 {
            commands.entity(entity).remove::<Tween<f32>>();
        }
    }
}

/// Advance every `Tween<Color>` and write the mixed color into the
/// entity's `BackgroundColor`. Removes the tween component when complete.
pub fn advance_color_tweens(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Tween<Color>, &mut BackgroundColor)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tween, mut bg) in &mut q {
        tween.elapsed += dt;
        let raw = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
        let eased = tween.easing.sample(raw).unwrap_or(raw);
        bg.0 = tween.start.mix(&tween.end, eased);
        if raw >= 1.0 {
            commands.entity(entity).remove::<Tween<Color>>();
        }
    }
}

/// Advance every `Tween<Val>` and write the interpolated value into the
/// entity's `Node.left`. `start` and `end` must both be `Val::Px(_)`;
/// debug builds assert, release falls back to treating other variants as 0px.
pub fn advance_val_tweens(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Tween<Val>, &mut Node)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tween, mut node) in &mut q {
        tween.elapsed += dt;
        let raw = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
        let eased = tween.easing.sample(raw).unwrap_or(raw);
        let start_px = px_of(tween.start);
        let end_px = px_of(tween.end);
        node.left = Val::Px(start_px.lerp(end_px, eased));
        if raw >= 1.0 {
            commands.entity(entity).remove::<Tween<Val>>();
        }
    }
}

fn px_of(v: Val) -> f32 {
    match v {
        Val::Px(x) => x,
        other => {
            debug_assert!(false, "Tween<Val> requires Val::Px, got {other:?}");
            0.0
        }
    }
}
