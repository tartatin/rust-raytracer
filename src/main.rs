use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

use std::thread;

use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;

use std::f64;

use std::time::SystemTime;
use std::time::Duration;

/**************************************************************************************************/

fn proute() {
    println!("Hello, world!");

    let connection = sqlite::open(":memory:").unwrap();

    let path = Path::new(r"/home/fabien/out.png");
    let file = File::create(path).unwrap();
    let ref mut buffer = BufWriter::new(file);

    let mut encoder = png::Encoder::new(buffer, 2, 2);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();

    let data: [u8; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    writer.write_image_data(&data).unwrap();

    connection
        .execute("CREATE TABLE coucouille (name TEXT, age INTEGER);")
        .unwrap();
}

/**************************************************************************************************/

#[derive(Copy, Clone)]
struct Vector {
    x: f64,
    y: f64,
    z: f64,
}

/**************************************************************************************************/

impl Vector {
    fn length(&self) -> f64 {
        return (self.x*self.x + self.y*self.y + self.z*self.z).sqrt();
    }

    fn length_square(&self) -> f64 {
        return self.x*self.x + self.y*self.y + self.z*self.z;
    }

    fn unit(&self) -> Vector {
        return self * (1.0 / self.length());
    }

    fn dot(&self, rhs: &Vector) -> f64 {
        return self.x*rhs.x + self.y*rhs.y + self.z*rhs.z;
    }

    fn cross(&self, rhs: &Vector) -> Vector {
        return Vector{
            x: self.y*rhs.z - self.z*rhs.y,
            y: self.z*rhs.x - self.x*rhs.z,
            z: self.x*rhs.y - self.y*rhs.x,
        };
    }
}

/**************************************************************************************************/

impl Add<&Vector> for &Vector {
    type Output = Vector;
    fn add(self, _rhs: &Vector) -> Vector {
        let result = Vector{
            x: self.x + _rhs.x,
            y: self.y + _rhs.y,
            z: self.z + _rhs.z,
        };
        return result;
    }
}

/**************************************************************************************************/

impl Sub<&Vector> for &Vector {
    type Output = Vector;
    fn sub(self, _rhs: &Vector) -> Vector {
        let result = Vector{
            x: self.x - _rhs.x,
            y: self.y - _rhs.y,
            z: self.z - _rhs.z,
        };
        return result;
    }
}

/**************************************************************************************************/

impl Mul<f64> for &Vector {
    type Output = Vector;
    fn mul(self, _rhs: f64) -> Vector {
        let result = Vector{
            x: self.x * _rhs,
            y: self.y * _rhs,
            z: self.z * _rhs,
        };
        return result;
    }
}

/**************************************************************************************************/

#[derive(Copy, Clone)]
struct Rgb {
    r: f64,
    g: f64,
    b: f64
}

impl Mul<f64> for &Rgb {
    type Output = Rgb;

    fn mul(self, factor: f64) -> Rgb {
        return Rgb{
            r: self.r * factor,
            g: self.g * factor,
            b: self.b * factor,
        };
    }
}

/**************************************************************************************************/

struct Body {
    position: Vector,
    velocity: Vector,
    radius: f64,
    mass: f64,
    color: Rgb,
}

/**************************************************************************************************/

struct Universe {
    central_body: Body,
    satellite: Body,
}

/**************************************************************************************************/

struct Camera {
    position: Vector,
    direction: Vector,
    up: Vector,
    hfov: f64,
}

/**************************************************************************************************/

fn create_universe() -> Universe {
    let earth_radius = 6371.0e3;

    let universe = Universe{
        central_body: Body {
            position: Vector{
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            velocity: Vector{
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            radius: earth_radius,
            mass: 5.972e24,
            color: Rgb{
                r: 1.0,
                g: 0.0,
                b: 0.0,
            }
        },

        satellite: Body {
            position: Vector {
                x: earth_radius*1.4,
                y: 0.0,
                z: 0.0,
            },
            velocity: Vector{
                x: 0.0,
                y: -6.15432014e3,
                z: 3e3,
            },
            radius: 1e6,
            mass: 500.0,
            color: Rgb{
                r: 0.0,
                g: 1.0,
                b: 0.0,
            }
        }
    };

    return universe;
}

/**************************************************************************************************/

fn update_universe(universe: &mut Universe, step_s: f64) {
    const G: f64 = 6.6743e-11;

    // Calcul de la distance entre les deux corps
    let diff: Vector = &universe.central_body.position - &universe.satellite.position;
    let distance_square = diff.length_square();

    // Calcul de l'effort gravitationnel
    let force_value: f64 = - &G * &universe.central_body.mass * &universe.satellite.mass / distance_square;
    let force_vec = &diff.unit() * force_value;

    // Intégration de la force sur la durée depuis la dernière boucle
    let acc_vec = &force_vec * (-1.0 / &universe.satellite.mass);
    universe.satellite.velocity = &universe.satellite.velocity + &(&acc_vec * step_s);
    universe.satellite.position = &universe.satellite.position + &(&universe.satellite.velocity * step_s);
}

/**************************************************************************************************/

fn deg_to_rad(deg: f64) -> f64 {
    return deg * f64::consts::PI / 180.0;
}

/**************************************************************************************************/

fn create_camera() -> Camera {
    let camera = Camera{
        position: Vector{x: 0.0, y: -1e8, z: 0.0},
        direction: Vector{x: 0.0, y: 1.0, z: 0.0},
        up: Vector{x: 0.0, y: 0.0, z: 1.0},
        hfov: deg_to_rad(20.0)
    };

    return camera;
}

/**************************************************************************************************/

struct BodySDF<'a> {
    body: &'a Body,
    previous_distance: f64,
    name: &'a str,
}

/**************************************************************************************************/

fn raycast(universe: &Universe, origin: &Vector, ray: &Vector) -> Rgb {
    // On fait du raymarching jusqu'à croiser un corps
    // TODO: pour l'instant on ne considère que le corps central

    let mut bodies: [BodySDF; 2] = [
        BodySDF{
            body: &universe.central_body,
            previous_distance: f64::MAX,
            name: "Terre"
        },
        BodySDF{
            body: &universe.satellite,
            previous_distance: f64::MAX,
            name: "Lune"
        }
    ];

    const PROXIMITY_THRESHOLD: f64 = 1.0; // 1 mètre de seuil de détection
    let mut position: Vector = *origin;
    let mut steps_count = 0;

    // Indique si on est en train de s'éloigner de TOUS les corps
    loop {
        let mut leaving: bool = true;
        steps_count += 1;

        let mut closest_distance = f64::MAX;

        for body_sdf in &mut bodies {
            let distance = (&position - &body_sdf.body.position).length() - body_sdf.body.radius;

            // Si on s'éloigne de l'objet, c'est foutu pour cet objet
            if distance > body_sdf.previous_distance {
                continue;
            } else {
                // On sait qu'au moins pour un objet, on n'est pas ne train de nous éloigner
                leaving = false;
            }

            // On conserve la distance à l'objet courant, pour pouvoir détermienr au cycle suivant si on s'éloigne
            body_sdf.previous_distance = distance;

            if distance > PROXIMITY_THRESHOLD {
                // On n'a pas encore croisé la bounding sphere
                closest_distance = f64::min(closest_distance, distance);
            } else {
                // On est dans la bounding sphere ou vraiment pas loin
                let attack = ray.dot(&(&position - &body_sdf.body.position).unit()).abs();
                let complexity = (steps_count as f64) / 50.0;

                return &body_sdf.body.color * attack;
            }
        }

        if leaving {
            return Rgb{
                r: 0.0,
                g: 0.0,
                b: 0.0,
            };
        }

        // On fait avancer le rayon
        position = &position + &(ray * (0.5*closest_distance));
    }
}

/**************************************************************************************************/

fn render_camera(universe: &Universe, camera: &Camera, frame_counter: u64) {
    const WIDTH: usize = 640;
    const HEIGHT: usize = 480;

    let focal: f64 = 1.0;

    let z_vec: Vector = camera.direction.unit();
    let x_vec: Vector = (&camera.up.unit() * -1.0).cross(&z_vec);
    let y_vec: Vector = z_vec.cross(&x_vec);

    let mut image_data: [u8; 3* WIDTH * HEIGHT] = [0; 3* WIDTH * HEIGHT];

    for y in 0..HEIGHT-1 {
        for x in 0..WIDTH-1 {
            let half_hfov = camera.hfov*0.5;
            let vfov = (HEIGHT as f64)/(WIDTH as f64) * camera.hfov;
            let half_vfov = vfov * 0.5;

            let h_angle = -half_hfov + camera.hfov * (x as f64) / ((WIDTH -1) as f64);
            let v_angle = -half_vfov +        vfov * (y as f64) / ((HEIGHT -1) as f64);

            let ray_x = &x_vec * (focal * h_angle.tan());
            let ray_y = &y_vec * (focal * v_angle.tan());
            let ray_z = &z_vec * focal;

            let ray: Vector = (&(&ray_z + &ray_x) + &ray_y);

            let color_f = raycast(&universe,&camera.position, &ray);
            let color_u = Rgb {
                r: color_f.r * 255.0,
                g: color_f.g * 255.0,
                b: color_f.b * 255.0,
            };

            // On a la couleur RGB de notre pixel !
            let image_pixel = 3*(y * WIDTH + x);
            image_data[image_pixel + 0] = color_u.r as u8;
            image_data[image_pixel + 1] = color_u.g as u8;
            image_data[image_pixel + 2] = color_u.b as u8;
        }
    }

    // Enregistrement de l'image
    let path_str = format!(r"/home/fabien/out_{frame_counter}.png");
    let path = Path::new(&path_str);
    let file = File::create(path).unwrap();
    let ref mut buffer = BufWriter::new(file);

    let mut encoder = png::Encoder::new(buffer,
                                        WIDTH as u32,
                                        HEIGHT as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&image_data).unwrap();
}

/**************************************************************************************************/

fn main()
{
    // Création de l'univers
    let mut universe = create_universe();

    // Création de la caméra
    let mut camera = create_camera();

    let mut frame_counter = 0;
    let mut last = SystemTime::now();
    loop {
        // Pause dans la matrice
        //let sleep_duration = Duration::from_millis(1000);
        //thread::sleep(sleep_duration);

        // Calcul de la période
        //let now = SystemTime::now();
        //let delta_ns = now.duration_since(last).unwrap().as_nanos();
        //let delta_s = (delta_ns as f64) * 1e-9;
        //last = now;
        let delta_s = 1.0;

        // Mise à jour de l'univers
        update_universe(&mut universe, delta_s);

        // Rendu d'une frame
        if frame_counter % (30) == 0 {
            println!("Writing frame {frame_counter}...");
            render_camera(&universe, &camera, frame_counter);
        }
        frame_counter += 1;
    }

    //println!("{a}");
}