pub const fn factorial(x: u64) -> u64 {
	let mut result: u64 = 1;
	let mut i: u64 = 1;
	while i <= x {
		result *= i;
		i += 1;
	}
	result
}

pub const fn factorial_reciprocal(x: u64) -> f64 {
	1.0 / (factorial(x) as f64)
}

fn _sin(x: f64) -> f64 {
	// Maclaurin series
	let x2 = x * x;
	let x3 = x2 * x;
	let x5 = x3 * x2;
	let x7 = x5 * x2;
	let x9 = x7 * x2;

	x - factorial_reciprocal(3) * x3 + factorial_reciprocal(5) * x5 - factorial_reciprocal(7) * x7
		+ factorial_reciprocal(9) * x9
}

pub fn sin(mut x: f64) -> f64 {
	let pi_over_2 = core::f64::consts::FRAC_PI_2;
	// Tau is 2π
	let tau = core::f64::consts::TAU;
	// Need to split the input so it's between 0 & π/2 (so approximation is valid)
	while x >= tau {
		x -= tau;
	}
	// Now x <= 2π
	// This switches the sign if π < x <= 2π
	let multiplier = if x > core::f64::consts::PI {
		x -= core::f64::consts::PI;
		-1.0
	} else {
		1.0
	};
	debug_assert!(x < core::f64::consts::PI);
	multiplier as u8 as f64
		* if x <= pi_over_2 {
			_sin(x)
		} else {
			// If π/2 < x <= π
			_sin(core::f64::consts::PI - x)
		}
}
