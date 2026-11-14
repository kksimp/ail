// Rust twin of raytrace.ail.
#[derive(Clone, Copy)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Clone, Copy)]
struct Sphere {
    c: Vec3,
    r: f64,
    col: Vec3,
    refl: f64,
}

fn v(x: f64, y: f64, z: f64) -> Vec3 { Vec3 { x, y, z } }
fn add(a: Vec3, o: Vec3) -> Vec3 { v(a.x + o.x, a.y + o.y, a.z + o.z) }
fn sub(a: Vec3, o: Vec3) -> Vec3 { v(a.x - o.x, a.y - o.y, a.z - o.z) }
fn scale(a: Vec3, k: f64) -> Vec3 { v(a.x * k, a.y * k, a.z * k) }
fn dot(a: Vec3, o: Vec3) -> f64 { a.x * o.x + a.y * o.y + a.z * o.z }
fn unit(a: Vec3) -> Vec3 { let k = 1.0 / dot(a, a).sqrt(); scale(a, k) }

fn hit(s: &Sphere, o: Vec3, d: Vec3) -> f64 {
    let oc = sub(o, s.c);
    let b = dot(oc, d);
    let c = dot(oc, oc) - s.r * s.r;
    let disc = b * b - c;
    if disc < 0.0 { return -1.0; }
    let sq = disc.sqrt();
    let t = -b - sq;
    if t > 0.001 { return t; }
    let t2 = -b + sq;
    if t2 > 0.001 { return t2; }
    -1.0
}

fn trace(spheres: &[Sphere; 4], light: Vec3, o: Vec3, d: Vec3, depth: i32) -> Vec3 {
    let mut best = -1.0f64;
    let mut idx: i32 = -1;
    for i in 0..4 {
        let t = hit(&spheres[i], o, d);
        if t > 0.0 && (best < 0.0 || t < best) { best = t; idx = i as i32; }
    }
    if idx < 0 {
        let k = 0.5 * (d.y + 1.0);
        return add(scale(v(1.0, 1.0, 1.0), 1.0 - k), scale(v(0.5, 0.7, 1.0), k));
    }
    let s = spheres[idx as usize];
    let p = add(o, scale(d, best));
    let n = unit(sub(p, s.c));
    let mut base = s.col;
    if idx == 3 {
        let parity = ((p.x + 100.0) as i32 + (p.z + 100.0) as i32) % 2;
        if parity == 0 { base = v(0.2, 0.2, 0.2); }
    }
    let mut lit = false;
    for i in 0..4 { if hit(&spheres[i], p, light) > 0.0 { lit = true; } }
    let mut diff = dot(n, light);
    if diff < 0.0 || lit { diff = 0.0; }
    let mut col = scale(base, 0.15 + 0.85 * diff);
    if depth < 2 && s.refl > 0.0 {
        let r = sub(d, scale(n, 2.0 * dot(d, n)));
        let rc = trace(spheres, light, p, r, depth + 1);
        col = add(scale(col, 1.0 - s.refl), scale(rc, s.refl));
    }
    col
}

fn chan(val: f64) -> i64 {
    let mut c = val;
    if c < 0.0 { c = 0.0; }
    if c > 1.0 { c = 1.0; }
    (c * 255.0) as i64
}

fn main() {
    let n: i64 = std::env::args().nth(1).map(|a| a.parse().unwrap_or(0)).unwrap_or(400);
    let spheres = [
        Sphere { c: v(0.0, 0.0, -5.0), r: 1.0, col: v(0.9, 0.2, 0.2), refl: 0.3 },
        Sphere { c: v(2.0, 0.2, -6.0), r: 1.2, col: v(0.2, 0.8, 0.3), refl: 0.5 },
        Sphere { c: v(-2.2, -0.3, -4.5), r: 0.7, col: v(0.3, 0.3, 0.9), refl: 0.2 },
        Sphere { c: v(0.0, -1001.0, -5.0), r: 1000.0, col: v(0.8, 0.8, 0.8), refl: 0.25 },
    ];
    let light = unit(v(0.5, 1.0, 0.3));
    let eye = v(0.0, 0.0, 0.0);
    let mut cks = 0i64;
    let mut total = 0i64;
    for j in 0..n {
        for i in 0..n {
            let u = (i as f64 + 0.5) / n as f64 * 2.0 - 1.0;
            let vv = 1.0 - (j as f64 + 0.5) / n as f64 * 2.0;
            let d = unit(v(u, vv, -1.5));
            let c = trace(&spheres, light, eye, d, 0);
            let (r, g, b) = (chan(c.x), chan(c.y), chan(c.z));
            total += r + g + b;
            cks = (cks * 31 + r * 65536 + g * 256 + b) % 1000000007;
        }
    }
    println!("size {}\ntotal {}\nchecksum {}", n, total, cks);
}
