use tardis::tle::TLE;
use tardis::traits::Observable;

use tardis::utils::{Coordinates, Observer};

fn main() -> Result<(), String> {
    let tle_lines = vec![
        "ISS (ZARYA)                                                          ".as_bytes(),
        "1 25544U 98067A   21288.70144628  .00006635  00000-0  12985-3 0  9991".as_bytes(),
        "2 25544  51.6430 106.8285 0003768 107.2156 352.5939 15.48692786307278".as_bytes(),
    ];
    /*let tle_lines = vec![
        "deb Ariane                                                           ".as_bytes(),
        "1 25543U 88109K   21289.14855083 -.00000085  00000-0  56178-3 0  9995".as_bytes(),
        "2 25543   6.5884 186.8092 7180051 173.3283 207.9933  2.29560923197418".as_bytes(),
    ];*/
    let tle_lines = vec![
        "ISS (ZARYA)                                                          ".as_bytes(),
        "1 25544U 98067A   21316.58314353 -.00007551  00000-0 -13101-3 0  9994".as_bytes(),
        "2 25544  51.6442 328.9484 0004731 186.1225 318.0089 15.48559922311590".as_bytes(),
    ];

    let mont_royal_coordinates = Coordinates::new(
        45.508888,
        -73.561668,
    );

    let satellite = match TLE::from_lines(&tle_lines[1], &tle_lines[2], &tle_lines[0]) {
        Ok(s) => s,
        Err(err) => {
            return Err("Cannot parse TLE: ".to_owned() + &err);
        }
    };

    println!("TLE: {}", satellite);

    println!("Earth satellite name: {}", satellite.name());

    let observer = Observer::new(mont_royal_coordinates);

    let obs = match satellite.observation(&observer) {
        Ok(o) => o,
        Err(e) => return Err(e),
    };

    println!("Current observation: {}", obs);

    Ok(())
}
