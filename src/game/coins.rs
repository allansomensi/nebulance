//! Coins: the currency of the shop. They come in six values (10, 25, 50, 100, 200 and 500),
//! drop only now and then, and the bigger ones are rare. Deeper levels pay a little better.

use super::*;

/// The coin values, smallest first.
pub const DENOMS: [u32; 6] = [10, 25, 50, 100, 200, 500];

/// One coin for a defeated enemy, or none. `chance` is the base drop chance.
pub(super) fn roll_coin(chance: f32, tier: u32) -> Option<u32> {
    if gen_range(0.0, 1.0) >= chance {
        return None;
    }
    // weights for 10, 25, 50, 100, 200, 500; later tiers shift towards bigger coins
    let t = tier.min(14) as f32;
    let w = [
        60.0 - t * 2.5,
        25.0,
        10.0 + t * 0.8,
        4.0 + t * 0.6,
        0.8 + t * 0.25,
        0.2 + t * 0.08,
    ];
    let total: f32 = w.iter().sum();
    let mut r = gen_range(0.0, total);
    for (i, x) in w.iter().enumerate() {
        if r < *x {
            return Some(DENOMS[i]);
        }
        r -= x;
    }
    Some(10)
}

/// Splits an amount into as few coins as possible (at most `max` coins).
pub(super) fn split(mut total: u32, max: usize) -> Vec<u32> {
    let mut out = vec![];
    for d in DENOMS.iter().rev() {
        while total >= *d && out.len() < max {
            out.push(*d);
            total -= d;
        }
    }
    out
}

impl Game {
    /// Drops coins worth `total` (rounded down to whole coins).
    pub(super) fn spawn_coins(&mut self, pos: Vec2, total: u32) {
        for v in split(total, 8) {
            self.drops.push(Drop {
                pos,
                vel: vec2(gen_range(-70.0, 70.0), gen_range(-140.0, -40.0)),
                kind: DropKind::Coin(v),
                age: 0.0,
            });
        }
    }
}

fn coin_colors(v: u32) -> (Color, Color, f32) {
    match v {
        10 => (
            Color::from_rgba(196, 116, 52, 255),
            Color::from_rgba(240, 170, 100, 255),
            5.0,
        ),
        25 => (
            Color::from_rgba(170, 176, 190, 255),
            Color::from_rgba(235, 240, 250, 255),
            5.5,
        ),
        50 => (
            Color::from_rgba(214, 160, 30, 255),
            Color::from_rgba(255, 226, 110, 255),
            6.0,
        ),
        100 => (
            Color::from_rgba(40, 170, 100, 255),
            Color::from_rgba(130, 240, 170, 255),
            6.5,
        ),
        200 => (
            Color::from_rgba(50, 110, 230, 255),
            Color::from_rgba(140, 190, 255, 255),
            7.0,
        ),
        _ => (
            Color::from_rgba(150, 70, 220, 255),
            Color::from_rgba(230, 170, 255, 255),
            8.0,
        ),
    }
}

/// Draws a spinning coin of value `v` centered at `p`; `scale` multiplies its size.
pub fn draw_coin(p: Vec2, v: u32, time: f32, scale: f32) {
    let (dark, light, r) = coin_colors(v);
    let r = r * scale;
    // the spin squashes the coin sideways
    let spin = ((time * 5.0 + p.x * 0.05).sin().abs() * 0.75 + 0.25).max(0.25);
    let w = r * spin;
    draw_circle(
        p.x,
        p.y,
        r * 1.9,
        Color::new(light.r, light.g, light.b, 0.12),
    );
    draw_ellipse(
        p.x,
        p.y,
        w + 1.2,
        r + 1.2,
        0.0,
        Color::from_rgba(20, 14, 30, 255),
    );
    draw_ellipse(p.x, p.y, w, r, 0.0, dark);
    draw_ellipse(
        p.x - w * 0.15,
        p.y - r * 0.15,
        w * 0.62,
        r * 0.62,
        0.0,
        light,
    );
    if v >= 500 {
        let a = (time * 9.0).sin().abs();
        let c = Color::new(1.0, 1.0, 1.0, 0.4 + a * 0.6);
        draw_line(p.x - r * 1.4, p.y, p.x + r * 1.4, p.y, 1.0, c);
        draw_line(p.x, p.y - r * 1.4, p.x, p.y + r * 1.4, 1.0, c);
    }
    if spin > 0.7 && scale >= 1.0 {
        let label = v.to_string();
        let sz = if v >= 100 { 7.0 } else { 8.0 } * scale;
        crate::ui::text_c(
            &label,
            p.x,
            p.y + sz * 0.38,
            sz,
            Color::from_rgba(20, 14, 30, 255),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_uses_real_coins() {
        assert_eq!(split(735, 8), vec![500, 200, 25, 10]);
        assert_eq!(split(5, 8), Vec::<u32>::new());
        assert!(split(10_000, 8).len() == 8);
    }
}
