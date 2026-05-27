use libm::cosf;
use libm::sinf;
use sola_raylib::prelude::*;

const DIST: f32 = 1.8;
const ANTIPODAL: [(i32, i32); 4] = [(0, 1), (2, 3), (4, 5), (6, 7)];

struct RotPlane {
    a: usize,
    b: usize,
    speed: f32,
}

fn rotate4d(v: Vector4, a1: usize, a2: usize, angle: f32) -> Vector4 {
    let mut vals: [f32; 4] = [v.x, v.y, v.z, v.w];
    let c = cosf(angle);
    let s = sinf(angle);
    let n1: f32 = vals[a1] * c - vals[a2] * s;
    let n2: f32 = vals[a1] * s + vals[a2] * c;
    vals[a1] = n1;
    vals[a2] = n2;
    Vector4::new(vals[0], vals[1], vals[2], vals[3])
}

fn project(v: Vector4) -> Vector3 {
    let f: f32 = 1.0 / (DIST - v.w);
    Vector3::new(v.x * f, v.y * f, v.z * f)
}

fn color_lerp(a: Color, b: Color, mut t: f32) -> Color {
    t = t.clamp(0.0, 1.0);
    Color::new(
        (a.r as f32 + (b.r as f32 - a.r as f32) * t) as u8,
        (a.g as f32 + (b.g as f32 - a.g as f32) * t) as u8,
        (a.b as f32 + (b.b as f32 - a.b as f32) * t) as u8,
        (a.a as f32 + (b.a as f32 - a.a as f32) * t) as u8,
    )
}

fn main() {
    let verts_rest: Vec<Vector4> = vec![
        Vector4::new(1.0, 0.0, 0.0, 0.0),
        Vector4::new(-1.0, 0.0, 0.0, 0.0),
        Vector4::new(0.0, 1.0, 0.0, 0.0),
        Vector4::new(0.0, -1.0, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 1.0, 0.0),
        Vector4::new(0.0, 0.0, -1.0, 0.0),
        Vector4::new(0.0, 0.0, 0.0, 1.0),
        Vector4::new(0.0, 0.0, 0.0, -1.0),
    ];

    let is_antipodal = |i: i32, j: i32| -> bool {
        ANTIPODAL
            .iter()
            .any(|&(p1, p2)| (p1 == i && p2 == j) || (p1 == j && p2 == i))
    };

    let mut edges: Vec<(i32, i32)> = Vec::new();
    let mut faces: Vec<[i32; 3]> = Vec::new();

    for i in 0..8_i32 {
        for j in (i + 1)..8_i32 {
            if !is_antipodal(i, j) {
                edges.push((i, j));
            }
        }
    }

    for skip in 0..4 {
        let mut chosen: Vec<[i32; 2]> = Vec::new();
        for k in 0..4 {
            if k != skip {
                let p = ANTIPODAL[k];
                chosen.push([p.0, p.1]);
            }
        }
        for mask in 0..8_usize {
            faces.push([
                chosen[0][(mask >> 0) & 1],
                chosen[1][(mask >> 1) & 1],
                chosen[2][(mask >> 2) & 1],
            ]);
        }
    }

    let planes: Vec<RotPlane> = vec![
        RotPlane {
            a: 0,
            b: 3,
            speed: 0.31,
        },
        RotPlane {
            a: 1,
            b: 3,
            speed: 0.47,
        },
        RotPlane {
            a: 2,
            b: 3,
            speed: 0.61,
        },
        RotPlane {
            a: 0,
            b: 1,
            speed: 0.19,
        },
        RotPlane {
            a: 1,
            b: 2,
            speed: 0.37,
        },
        RotPlane {
            a: 0,
            b: 2,
            speed: 0.53,
        },
    ];

    let win_w: i32 = 1280;
    let win_h: i32 = 720;

    let (mut rl, thread) = sola_raylib::init()
        .size(win_w, win_h)
        .title("16-cell")
        .highdpi()
        .build();
    rl.set_target_fps(60);

    let cam = Camera3D::perspective(
        Vector3::new(0.0, 0.0, 4.5),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        50.0,
    );

    let col_far: Color = Color::new(255, 70, 180, 255);
    let col_near: Color = Color::new(70, 200, 255, 255);

    let mut sim_time: f32 = 0.0;
    let mut speed: f32 = 1.0;
    let mut paused: bool = false;
    let mut show_faces: bool = true;

    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            paused = !paused;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F) {
            show_faces = !show_faces;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            sim_time = 0.0;
            speed = 1.0;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            speed = (speed + 0.2).min(5.0);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            speed = (speed - 0.2).max(0.1);
        }

        if !paused {
            sim_time += rl.get_frame_time() * speed;
        }
        let mut rot: Vec<Vector4> = verts_rest.clone();
        for v in rot.iter_mut() {
            for p in &planes {
                *v = rotate4d(*v, p.a, p.b, sim_time * p.speed);
            }
        }

        let p3: Vec<Vector3> = rot.iter().map(|v| project(*v)).collect();

        let w_min = rot.iter().map(|v| v.w).fold(f32::INFINITY, f32::min);
        let w_max = rot.iter().map(|v| v.w).fold(f32::NEG_INFINITY, f32::max);

        let w_norm = |w: f32| -> f32 {
            if w_max > w_min {
                (w - w_min) / (w_max - w_min)
            } else {
                0.5
            }
        };

        rl.draw(&thread, |mut d| {
            d.clear_background(Color::BLACK);

            d.draw_mode3D(cam, |mut d2, _cam| {
                if show_faces {
                    let mut sf: Vec<(f32, [usize; 3])> = faces
                        .iter()
                        .map(|f| {
                            let i = [f[0] as usize, f[1] as usize, f[2] as usize];
                            let z = (p3[i[0]].z + p3[i[1]].z + p3[i[2]].z) / 3.0;
                            (z, i)
                        })
                        .collect();

                    sf.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                    for (_, f) in &sf {
                        let w_avg = (rot[f[0]].w + rot[f[1]].w + rot[f[2]].w) / 3.0;
                        let mut fc = color_lerp(col_near, col_far, w_norm(w_avg));
                        fc.a = 35;
                        d2.draw_triangle3D(p3[f[0]], p3[f[1]], p3[f[2]], fc);
                        d2.draw_triangle3D(p3[f[2]], p3[f[1]], p3[f[0]], fc);
                    }
                }
                for &(i, j) in &edges {
                    let (i, j) = (i as usize, j as usize);
                    let w_avg = (rot[i].w + rot[j].w) * 0.5;
                    let mut ec = color_lerp(col_near, col_far, w_norm(w_avg));
                    ec.a = 220;
                    d2.draw_line_3D(p3[i], p3[j], ec);
                }
                for i in 0..8 {
                    let vc = color_lerp(col_near, col_far, w_norm(rot[i].w));
                    d2.draw_sphere(p3[i], 0.046, vc);
                }
            });
            d.draw_fps(win_w - 88, win_h - 24);
            let ui_x = 12;
            let line = |i: i32| 12 + i * 20;
            let speed_label = format!("Speed: {:.1}x", speed);
            d.draw_text(&speed_label, ui_x, line(0), 18, Color::WHITE);

            let pause_str = if paused { "[PAUSED]" } else { "" };
            d.draw_text(pause_str, ui_x, line(2), 16, Color::new(255, 200, 50, 255));

            let binds: &[(&str, &str)] = &[
                ("SPACE", "pause"),
                ("Left/Right", "speed"),
                ("F", "faces on/off"),
                ("R", "reset"),
            ];
            let by = line(3) + 8;
            d.draw_text("Controls", ui_x, by, 16, Color::new(180, 180, 180, 200));
            d.draw_line(
                ui_x,
                by + 18,
                ui_x + 110,
                by + 18,
                Color::new(100, 100, 100, 180),
            );
            for (i, (key, desc)) in binds.iter().enumerate() {
                let y = by + 22 + i as i32 * 18;
                d.draw_text(key, ui_x, y, 15, Color::new(70, 200, 255, 220));
                d.draw_text(desc, ui_x + 100, y, 15, Color::new(200, 200, 200, 200));
            }
        });
    }
}
