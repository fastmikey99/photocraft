//! Explicit raster-lattice round screen, observed from a Photoshop-authored bitmap.
//! This is an opt-in prototype, not an inference of Photoshop's frequency/angle rounding.
use std::collections::{BTreeMap, HashMap};

/// A ranked periodic screen with circular dots and complementary circular holes.
/// Coordinates refer to integer pixel indices; `origin` is a dot centre.
pub struct LatticeScreen {
    step: [i64; 2],
    origin: [i64; 2],
    determinant: i64,
    thresholds: HashMap<(i64, i64), f32>,
}

impl LatticeScreen {
    /// The perpendicular basis is (-step_y, step_x). Bound setup work independently of images.
    pub fn new(step: [i32; 2], origin: [i32; 2]) -> Result<Self, String> {
        let [a, b] = step.map(i64::from);
        if a.abs() > 256 || b.abs() > 256 {
            return Err("lattice components must be in -256..256".into());
        }
        let n = a * a + b * b;
        if !(4..=65536).contains(&n) {
            return Err("lattice squared length must be in 4..=65536".into());
        }
        let bound = a.abs() + b.abs();
        let mut sites = BTreeMap::new();
        for y in -bound..=bound {
            for x in -bound..=bound {
                let key = ((a * x + b * y).rem_euclid(n), (-b * x + a * y).rem_euclid(n));
                // Doubled signed residues keep odd-length lattices exact, without float ties.
                let u = (2 * key.0 + n).rem_euclid(2 * n) - n;
                let v = (2 * key.1 + n).rem_euclid(2 * n) - n;
                let score = if u.abs() + v.abs() < n { u * u + v * v } else { 2 * n * n - (n - u.abs()).pow(2) - (n - v.abs()).pow(2) };
                // Deterministic ties: lower image rows first, then rightmost pixels.
                // The ordering is a prototype convention, not claimed byte-exact Photoshop parity.
                sites.insert(key, (score, -(b * u + a * v), -(a * u - b * v)));
            }
        }
        let mut ranked: Vec<_> = sites.into_iter().collect();
        ranked.sort_by_key(|(key, score)| (*score, *key));
        let count = ranked.len() as f32;
        let thresholds = ranked.into_iter().enumerate().map(|(rank, (key, _))| (key, (rank as f32 + 0.5) / count)).collect();
        Ok(Self { step: [a, b], origin: origin.map(i64::from), determinant: n, thresholds })
    }

    /// Gray is white=1, black=0. Both endpoints are preserved at every sample depth.
    pub fn apply(&self, gray: &mut [f32], width: usize) {
        if width == 0 {
            return;
        }
        let [a, b] = self.step;
        let n = self.determinant;
        for (i, g) in gray.iter_mut().enumerate() {
            let x = (i % width) as i64 - self.origin[0];
            let y = (i / width) as i64 - self.origin[1];
            let key = ((a * x + b * y).rem_euclid(n), (-b * x + a * y).rem_euclid(n));
            if let Some(threshold) = self.thresholds.get(&key) {
                *g = if 1.0 - *g > *threshold { 0.0 } else { 1.0 };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn periodicity_phase_endpoints_and_coverage() {
        let screen = LatticeScreen::new([18, 8], [3, -6]).unwrap();
        assert_eq!(screen.thresholds.len(), 388);
        let mut data = vec![0.5; 194 * 194];
        screen.apply(&mut data, 194);
        assert_eq!(data.iter().filter(|&&g| g == 0.0).count(), 194 * 194 / 2);
        for y in 0..170 {
            for x in 8..170 {
                assert_eq!(data[y * 194 + x], data[(y + 8) * 194 + x + 18]);
                assert_eq!(data[y * 194 + x], data[(y + 18) * 194 + x - 8]);
            }
        }
        let shifted = LatticeScreen::new([18, 8], [4, -4]).unwrap();
        let mut moved = vec![0.5; data.len()];
        shifted.apply(&mut moved, 194);
        for y in 0..190 {
            for x in 0..190 {
                assert_eq!(data[y * 194 + x], moved[(y + 2) * 194 + x + 1]);
            }
        }
        for endpoint in [0.0, 1.0] {
            data.fill(endpoint);
            screen.apply(&mut data, 194);
            assert!(data.iter().all(|&g| g == endpoint));
        }
    }

    #[test]
    fn rejects_unbounded_work_and_accepts_odd_lattices() {
        for step in [[0, 0], [1, 0], [i32::MAX, i32::MAX], [257, 0]] {
            assert!(LatticeScreen::new(step, [0, 0]).is_err());
        }
        let s = LatticeScreen::new([3, 2], [i32::MIN, i32::MAX]).unwrap();
        assert_eq!(s.thresholds.len(), 13);
        s.apply(&mut [], 0);
        let mut ramp: Vec<_> = (0..260).map(|i| i as f32 / 259.0).collect();
        s.apply(&mut ramp, 13);
        assert!(ramp.iter().all(|&v| v == 0.0 || v == 1.0));
    }
}
