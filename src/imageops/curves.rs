use decoders::Image;
use std::cmp;

pub fn base(_: &Image, inb: &[f32]) -> Vec<f32> {
  let mut out: Vec<f32> = vec![0.0; inb.len() as usize];  

  let xs = [0.0, 0.050, 0.100, 0.450, 0.850, 1.0];
  let ys = [0.0, 0.055, 0.220, 0.770, 0.985, 1.0];
  let func = SplineFunc::new(&xs, &ys);

  for (i,val) in inb.iter().enumerate() {
    out[i] = func.interpolate(*val);
  }

  out
}


struct SplineFunc<'a> {
  xs: &'a [f32],
  ys: &'a [f32],
  c1s: Vec<f32>,
  c2s: Vec<f32>,
  c3s: Vec<f32>,
}

impl<'a> SplineFunc<'a> {
  // Monotone cubic interpolation code adapted from the Javascript example in Wikipedia
  fn new(xs: &'a[f32], ys: &'a[f32]) -> SplineFunc<'a> {
    if xs.len() != ys.len() { panic!("Different number of Xs and Ys for Spline"); }
    if xs.len() < 2 { panic!("Need at least 2 points for Spline"); }

	  // Get consecutive differences and slopes
    let mut dxs = Vec::new();
    let mut dys = Vec::new();
    let mut slopes = Vec::new();
    for i in 0..(xs.len()-1) {
      let dx = xs[i+1] - xs[i];
      let dy = ys[i+1] - ys[i];
      dxs.push(dx);
      dys.push(dy);
      slopes.push(dy/dx);
    }

	  // Get degree-1 coefficients
    let mut c1s = vec![slopes[0]];
    for i in 0..(dxs.len()-1) {
      let m = slopes[i];
      let next = slopes[i+1];
      if m*next <= 0.0 {
        c1s.push(0.0);
      } else {
        let dx = dxs[i];
        let dxnext = dxs[i+1];
        let common = dx + dxnext;
        c1s.push(3.0*common/((common+dxnext)/m + (common + dx)/next));
      }
    }
    c1s.push(slopes[slopes.len()-1]);

	  // Get degree-2 and degree-3 coefficients
    let mut c2s = Vec::new();
    let mut c3s = Vec::new();
    for i in 0..(c1s.len()-1) {
      let c1 = c1s[i];
      let slope = slopes[i];
      let invdx = 1.0 / dxs[i];
      let common = c1+c1s[i+1] - slope - slope;
      c2s.push((slope-c1-common)*invdx);
      c3s.push(common*invdx*invdx);
    }

    SplineFunc {
      xs: xs,
      ys: ys,
        c1s: c1s,
      c2s: c2s,
      c3s: c3s,
    }
  }

  fn interpolate(&self, val: f32) -> f32 {
		// Anything at or beyond the last value returns the last value
    let end = self.xs[self.xs.len()-1];
    if val >= end {
      return self.ys[self.ys.len()-1];
    }

		// Search for the interval x is in, returning the corresponding y if x is one of the original xs
    let mut low: isize = 0;
    let mut mid: isize;
    let mut high: isize = (self.c3s.len() - 1) as isize;

		while low <= high {
      mid = (low+high)/2;
      let xhere = self.xs[mid as usize];
      if xhere < val { low = mid + 1; }
      else if xhere > val { high = mid - 1; }
      else { return self.ys[mid as usize] }
		}
    let i = cmp::max(0, high) as usize;

		// Interpolate
    let diff = val - self.xs[i];

    self.ys[i] + self.c1s[i]*diff + self.c2s[i]*diff*diff + self.c3s[i]*diff*diff*diff
	}
}