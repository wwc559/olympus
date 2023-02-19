use clap::{arg, value_parser, Command};

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
}

//gravitational constant  m^3/(kg s^2)
const G: f64 = 6.67e-11;
const M_EARTH: f64 = 5.97e24;
const D_EARTH: f64 = 6.371e6;
const KARMAN_LINE: f64 = 100000.0;
// at -1000m it is 1.347
const DENSITY_AT_10KM: [f64; 11] = [
    1.22, 0.413, 8.89e-2, 1.84e-2, 4e-3, 1.03e-3, 3.1e-4, 8.3e-5, 1.85e-5, 4.12e-6, 0.00,
];

fn main() {
    let matches = cli().get_matches();
    let initial_distance = *matches.get_one::<f64>("distance").unwrap();
    let mut distance = initial_distance;
    let width = *matches.get_one::<f64>("width").unwrap();
    let mass = *matches.get_one::<f64>("mass").unwrap();
    let delta_t = *matches.get_one::<f64>("integration_time").unwrap();
    let mut velocity = 0.0;
    let mut t: f64 = 0.0;
    let table_limit = (DENSITY_AT_10KM.len() - 1) as f64 * 10000.0;
    println!("distance={}, width={}, mass={}", distance, width, mass);
    while distance - (velocity * delta_t) > 0.0 {
        let a_g = f_gravity(mass, M_EARTH, D_EARTH + distance) / mass;
        let a_d = if distance < table_limit {
            let index = distance as usize / 10000;
            f_drag(DENSITY_AT_10KM[index], velocity, width) / mass
        } else {
            0.0
        };
        velocity += (a_g - a_d) * delta_t;
        if ((t - t.floor()) < delta_t)
            && (distance < KARMAN_LINE || t < 20.0 || (t as i64 % (3600 * 6)) == 0)
        {
            println!(
                "{:.2} sec({:.2} days): v:{:.2} m/s ({:.2} mph), d:{:.2} m ({:.2} moonunits) ag:{:.2} ad:{:.2}",
                t,
                t as f64 / (60.0 * 60.0 * 24.0),
                velocity,
                velocity * 2.237,
                distance,
                distance / 384400000.0,
                a_g, a_d,
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
        "Precicely, after {:.2} seconds it was {:.2} m high moving at {:.2} m/s",
        t, distance, velocity
    );
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
