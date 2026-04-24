use color_science::types::Lab;
use color_science::delta_e::delta_e_2000;

fn main() {
    let lab1 = Lab { L: 50.0000, a: 2.6772, b: -79.7751 };
    let lab2 = Lab { L: 50.0000, a: 0.0000, b: -82.7485 };
    println!("pair1: {}", delta_e_2000(&lab1, &lab2));

    let lab1 = Lab { L: 50.0000, a: -1.1848, b: -84.8006 };
    let lab2 = Lab { L: 50.0000, a: 0.0000, b: -82.7485 };
    println!("pair2: {}", delta_e_2000(&lab1, &lab2));

    let lab1 = Lab { L: 50.0, a: 0.0, b: 0.0 };
    let lab2 = Lab { L: 90.0, a: 50.0, b: 50.0 };
    println!("large: {}", delta_e_2000(&lab1, &lab2));
}
