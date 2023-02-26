use clap::{arg, value_parser, ArgAction, Command};

fn cli() -> Command {
    Command::new("olympus")
        .about("Calculate the time an anvil takes to drop from olympus")
        .arg(
            arg!(-d --distance [DISTANCE] "distance to olympus (above earth surface)")
                .value_parser(value_parser!(f64))
                .default_value("573851000"),
        )
        .arg(
            arg!(-w --width [WIDTH] "width of the object (m) (presumes cube")
                .value_parser(value_parser!(f64))
                // default is a CLASSIC 110 Anivl
                .default_value("0.279"),
        )
        .arg(
            arg!(-m --mass [MASS] "mass of the object (kg)")
                .value_parser(value_parser!(f64))
                .default_value("117"),
        )
        .arg(
            arg!(-i --integration_time [T] "integration time (s)")
                .value_parser(value_parser!(f64))
                .default_value("0.01"),
        )
        .arg(arg!(-t --tartarus "calculate distance to tartarus").action(ArgAction::SetTrue))
}

//gravitational constant  m^3/(kg s^2)
const G: f64 = 6.67e-11;
const M_EARTH: f64 = 5.97e24;
// average radius
const R_EARTH: f64 = 6.367e6;
const KARMAN_LINE: f64 = 100000.0;

fn main() {
    let matches = cli().get_matches();
    let initial_distance = *matches.get_one::<f64>("distance").unwrap();
    let mut distance = initial_distance;
    let width = *matches.get_one::<f64>("width").unwrap();
    let mass = *matches.get_one::<f64>("mass").unwrap();
    let delta_t = *matches.get_one::<f64>("integration_time").unwrap();
    let tartarus = matches.get_flag("tartarus");
    let mut velocity = 0.0;
    let mut t: f64 = 0.0;
    let table_limit = (DENSITY_AT_10KM.len() - 1) as f64 * 10000.0;
    println!("distance={}, width={}, mass={}", distance, width, mass);
    while (tartarus && t < 9.0 * 24.0 * 3600.0) || (distance - (velocity * delta_t) > 0.0) {
        let m_earth = mass_of_earth(distance);
        let density = air_density(distance);
        let a_g = f_gravity(mass, m_earth, distance + R_EARTH) / mass;
        let a_d = if distance < table_limit {
            f_drag(density, velocity, width) / mass
        } else {
            0.0
        };
        velocity += (a_g - a_d) * delta_t;
        if ((t - t.floor()) < delta_t)
            && (t<20.0 || (distance > 0.0 && distance < 5.0* KARMAN_LINE) || (t as i64 % (3600 * 6)) == 0)
        {
            println!(
                "{:.2} sec({:.2} days): v:{:.2} m/s ({:.2} mph), d:{:.2} m ({:.2} moonunits) ag:{:.2} ad:{:.2} me:{:.2e} density:{:.2}",
                t,
                t as f64 / (60.0 * 60.0 * 24.0),
                velocity,
                velocity * 2.237,
                distance,
                distance / 384400000.0,
                a_g, a_d, m_earth, density
            );
        }
        distance -= velocity * delta_t;
        t += delta_t;
    }
    println!(
        "\nA {} kg anvil, dropped from {} km above the earth, will strike  after {:.2} days.",
        mass,
        initial_distance / 1000.0,
        t as f64 / (60.0 * 60.0 * 24.0)
    );
    println!(
        "Precicely, after {:.2} seconds it was {:.2} m above sea level, moving at {:.2} m/s",
        t, distance, velocity
    );
}

// at -1000m it is 1.347
const DENSITY_AT_10KM: [f64; 11] = [
    1.22, 0.413, 8.89e-2, 1.84e-2, 4e-3, 1.03e-3, 3.1e-4, 8.3e-5, 1.85e-5, 4.12e-6, 0.00,
];
fn air_density(elevation: f64) -> f64 {
    if elevation > KARMAN_LINE {
	0.0
    } else if elevation > 0.0 {
	let index = elevation as usize / 10000;
	//let p_sea_level = 101.325 / 98.06;
	//println!("{:.2} {:.2}", p_sea_level * (1.0 - 2.25577e-5 * elevation).powf(5.25588),DENSITY_AT_10KM[index]);
	DENSITY_AT_10KM[index]
    } else {
	let p_sea_level = 101325.0;	// in Pa 
	let p = p_sea_level * (1.0 - 2.25577e-5 * elevation).powf(5.25588);
	// ideal gas constant
	let r = 8.314;
	let t = 288.0;
	// molar mass of air kg/mol
	let m = 0.02897;
	(p * m) / ( r * t)
    }
}

fn volume_of_sphere(radius: f64) -> f64 {
    radius * radius * radius * std::f64::consts::PI * 4.0 / 3.0
}

fn mass_of_earth(elevation: f64) -> f64 {
    if elevation > 0.0 {
        M_EARTH
    } else {
        let density = M_EARTH / volume_of_sphere(R_EARTH);
        density * volume_of_sphere(R_EARTH + elevation).abs()
    }
}

/// force of gravity: ((m^3/(kg s^2)) kg kg / m m) = kg m/s^2
fn f_gravity(m1: f64, m2: f64, d: f64) -> f64 {
    (G * m1 * m2) / (d * d)
}

// presume anvil is a cube
fn f_drag(density: f64, velocity: f64, width: f64) -> f64 {
    0.5 * density * velocity * velocity * width * width * 1.09
}

#[allow(dead_code)]
fn f_stokes(viscosity: f64, radius: f64, velocity: f64) -> f64 {
    6.0 * std::f64::consts::PI * viscosity * radius * velocity
}
