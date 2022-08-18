use crate::position::Position;
use crate::units::f32::meter;
use crate::units::f32::Length;
use crate::units::vec2;

#[derive(Clone, Debug)]
pub struct Extents {
    pub x_min: Length,
    pub x_max: Length,
    pub y_min: Length,
    pub y_max: Length,
}

impl Extents {
    pub fn new(x_min: Length, x_max: Length, y_min: Length, y_max: Length) -> Self {
        debug_assert!(x_min < x_max);
        debug_assert!(y_min < y_max);
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    pub fn from_positions<'a>(positions: impl Iterator<Item = &'a vec2::Length>) -> Option<Self> {
        let mut x_min = None;
        let mut x_max = None;
        let mut y_min = None;
        let mut y_max = None;
        let update_min = |x: &mut Option<Length>, y: Length| {
            if x.is_none() || y < x.unwrap() {
                *x = Some(y);
            }
        };
        let update_max = |x: &mut Option<Length>, y: Length| {
            if x.is_none() || y > x.unwrap() {
                *x = Some(y);
            }
        };
        for pos in positions {
            update_min(&mut x_min, pos.x());
            update_max(&mut x_max, pos.x());
            update_min(&mut y_min, pos.y());
            update_max(&mut y_max, pos.y());
        }
        if x_min == x_max || y_min == y_max {
            None
        } else {
            Some(Self::new(x_min?, x_max?, y_min?, y_max?))
        }
    }

    pub fn contains(&self, pos: &Position) -> bool {
        self.x_min <= pos.0.x()
            && pos.0.x() < self.x_max
            && self.y_min <= pos.0.y()
            && pos.0.y() < self.y_max
    }

    pub fn get_quadrants(&self) -> [Self; 4] {
        let center_x = (self.x_min + self.x_max) * 0.5;
        let center_y = (self.y_min + self.y_max) * 0.5;
        let lower_left = Extents {
            x_min: self.x_min,
            x_max: center_x,
            y_min: self.y_min,
            y_max: center_y,
        };
        let lower_right = Extents {
            x_min: center_x,
            x_max: self.x_max,
            y_min: self.y_min,
            y_max: center_y,
        };
        let upper_right = Extents {
            x_min: center_x,
            x_max: self.x_max,
            y_min: center_y,
            y_max: self.y_max,
        };
        let upper_left = Extents {
            x_min: self.x_min,
            x_max: center_x,
            y_min: center_y,
            y_max: self.y_max,
        };
        [lower_left, lower_right, upper_right, upper_left]
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use crate::domain::Extents;
    use crate::units::f32::meter;
    use crate::units::f32::Length;
    use crate::units::vec2;

    fn assert_is_close(x: Length, y: f32) {
        const EPSILON: f32 = 1e-20;
        assert!((x - meter(y)).unwrap_value().abs() < EPSILON)
    }

    #[test]
    fn extent_quadrants() {
        let root_extents = Extents::new(meter(-1.0), meter(1.0), meter(-2.0), meter(2.0));
        let quadrants = root_extents.get_quadrants();
        assert_is_close(quadrants[0].x_min, -1.0);
        assert_is_close(quadrants[0].x_max, 0.0);
        assert_is_close(quadrants[0].y_min, -2.0);
        assert_is_close(quadrants[0].y_max, 0.0);

        assert_is_close(quadrants[1].x_min, 0.0);
        assert_is_close(quadrants[1].x_max, 1.0);
        assert_is_close(quadrants[1].y_min, -2.0);
        assert_is_close(quadrants[1].y_max, 0.0);

        assert_is_close(quadrants[2].x_min, 0.0);
        assert_is_close(quadrants[2].x_max, 1.0);
        assert_is_close(quadrants[2].y_min, 0.0);
        assert_is_close(quadrants[2].y_max, 2.0);

        assert_is_close(quadrants[3].x_min, -1.0);
        assert_is_close(quadrants[3].x_max, 0.0);
        assert_is_close(quadrants[3].y_min, 0.0);
        assert_is_close(quadrants[3].y_max, 2.0);
    }

    #[test]
    fn extent_from_positions() {
        let positions = &[
            vec2::meter(Vec2::new(1.0, 0.0)),
            vec2::meter(Vec2::new(-1.0, 0.0)),
            vec2::meter(Vec2::new(0.0, -2.0)),
            vec2::meter(Vec2::new(0.0, 2.0)),
        ];
        let extents = Extents::from_positions(positions.iter()).unwrap();
        assert_is_close(extents.x_min, -1.0);
        assert_is_close(extents.x_max, 1.0);
        assert_is_close(extents.y_min, -2.0);
        assert_is_close(extents.y_max, 2.0);
    }

    #[test]
    fn extent_from_positions_is_none_with_zero_positions() {
        assert!(Extents::from_positions([].iter()).is_none());
    }

    #[test]
    fn extent_from_positions_is_none_with_particles_at_same_positions() {
        let positions = &[
            vec2::meter(Vec2::new(1.0, 0.0)),
            vec2::meter(Vec2::new(1.0, 0.0)),
            vec2::meter(Vec2::new(1.0, 0.0)),
        ];
        assert!(Extents::from_positions(positions.iter()).is_none());
    }
}