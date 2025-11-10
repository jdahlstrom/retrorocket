use minifb::{Key, KeyRepeat, Scale, WindowOptions};
use std::ops::ControlFlow::Continue;
use std::time::Instant;

use re::prelude::*;

use re::core::util::Dims;
use re::geom::solids::{Build, Icosphere};
use re::prelude::clip::ClipVec;
use re_front::minifb::Window;
use re_front::Frame;

use entity::*;

mod entity;

const DIMS: Dims = (640, 360);

fn vertex_shader<P, A, Pt: Apply<P, Output = ClipVec>, At: Apply<A>>(
    v: Vertex<P, A>,
    (pos_tf, attr_tf): (&Pt, &At),
) -> Vertex<ClipVec, At::Output> {
    vertex(pos_tf.apply(&v.pos), attr_tf.apply(&v.attrib))
}

const MACHINE_GUN: Gun = Gun {
    cooldown: 0.05,
    muzzle_vel: 400.0,
    spread: polar(0.1, degs(1.5)),
    shots: 1,
    bullets: vec![],
};
const SHOTGUN: Gun = Gun {
    cooldown: 0.5,
    muzzle_vel: 300.0,
    spread: polar(0.2, degs(5.0)),
    shots: 20,
    bullets: vec![],
};
const CLAYMORE: Gun = Gun {
    cooldown: 1.0,
    muzzle_vel: 300.0,
    spread: polar(0.0, degs(360.0)),
    shots: 100,
    bullets: vec![],
};

fn main() {
    let mut win = Window::builder()
        .dims(DIMS)
        .title("Retrorocket")
        .options(WindowOptions {
            scale: Scale::X2,
            ..Default::default()
        })
        .target_fps(Some(120))
        .build()
        .unwrap();

    let player = Ship {
        pos: pt2(DIMS.0 as f32 / 2.0, DIMS.1 as f32 / 2.0),
        dir: vec2(0.0, -1.0),
        guns: vec![MACHINE_GUN.clone(), SHOTGUN.clone(), CLAYMORE.clone()],
        ..Ship::default()
    };
    let rock = Rock {
        pos: pt2(200.0, 100.0),
        vel: vec2(20.0, 7.0),
        mesh: Icosphere(1.0, 1).build(),
        ..Rock::default()
    };
    let mut level = Level { player, rock };

    let mut paused = false;
    let start = Instant::now();
    win.run(
        |Frame {
             t, dt, buf, win, ..
         }| {
            let w = &win.imp;
            let _t = t.as_secs_f32();
            let dt = dt.as_secs_f32();

            let plr = &mut level.player;

            plr.thrust = false;
            plr.acc = Vec2::zero();
            let mut target_rot = Angle::zero();
            if w.is_key_down(Key::Up) {
                plr.thrust();
            }
            if w.is_key_down(Key::Left) {
                target_rot = Ship::ROT_RATE;
            }
            if w.is_key_down(Key::Right) {
                target_rot = -Ship::ROT_RATE;
            }
            if w.is_key_down(Key::Space) {
                plr.fire();
            }
            if w.is_key_pressed(Key::P, KeyRepeat::No) {
                paused = !paused;
            }
            if w.is_key_pressed(Key::LeftSuper, KeyRepeat::No) {
                plr.guns.rotate_left(1);
            }
            if !paused {
                plr.rotate(target_rot, dt);
                plr.acc += vec2(0.0, 20.0); // Gravity

                level.update(dt);
            }

            // RENDER

            level.render(buf);
            Continue(())
        },
    );
    let stats = win.ctx.stats.borrow();
    let elapsed = start.elapsed();
    eprintln!(
        "time elapsed: {:.2?}, frames: {}, fps: {:.2}",
        elapsed,
        stats.frames,
        stats.frames / elapsed.as_secs_f32()
    );
}
