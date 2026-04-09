use std::{ops::ControlFlow::Continue, time::Instant};

use minifb::{Key, KeyRepeat, Scale, WindowOptions};

use re::prelude::*;

use re::core::{render::cam::Fov::Diagonal, render::clip::ClipVec, util::Dims};
use re::front::minifb::Window;
use re::geom::solids::{Build, Cone, Icosphere};

use entity::{Context, *};

mod entity;

const DIMS: Dims = (640, 360);

fn vertex_shader<P, A, Pt: Apply<P, Output = ClipVec>, At: Apply<A>>(
    v: Vertex<P, A>,
    (pos_tf, attr_tf): (&Pt, &At),
) -> Vertex<ClipVec, At::Output> {
    vertex(pos_tf.apply(&v.pos), attr_tf.apply(&v.attrib))
}

const MACHINE_GUN: Gun = Gun {
    muzzle_vel: 400.0,
    spread: polar(0.0, degs(1.5)),
    lifetime: 1.0,
    burst: Burst {
        shots: 1,
        interval: 0.0,
        cooldown: 0.05,
    },

    cooldown: 0.0,
    bullets: Emitter::new(),
};
const SHOTGUN: Gun = Gun {
    muzzle_vel: 300.0,
    spread: polar(0.1, degs(5.0)),
    lifetime: 2.0,
    burst: Burst {
        shots: 20,
        interval: 0.0,
        cooldown: 0.5,
    },

    cooldown: 0.0,
    bullets: Emitter::new(),
};
const CLAYMORE: Gun = Gun {
    muzzle_vel: 300.0,
    spread: polar(0.01, degs(120.0)),
    lifetime: 0.5,
    burst: Burst {
        shots: 100,
        interval: 0.0,
        cooldown: 1.0,
    },

    cooldown: 0.0,
    bullets: Emitter::new(),
};
const FLAMETHROWER: Gun = Gun {
    muzzle_vel: 50.0,
    spread: polar(0.3, degs(20.0)),
    lifetime: 1.0,
    burst: Burst {
        shots: 1000,
        interval: 0.01,
        cooldown: 1.0,
    },

    cooldown: 0.0,
    bullets: Emitter::new(),
};

fn main() {
    let mut win = Window::builder()
        .dims(DIMS)
        .title("Retrorocket")
        .options(WindowOptions {
            scale: Scale::X2,
            ..Default::default()
        })
        .target_fps(Some(0))
        .build()
        .unwrap();

    win.ctx.face_cull = None;

    let player = Ship {
        pos: pt2(0.0, 0.0),
        dir: Vec2::Y,
        guns: vec![
            MACHINE_GUN.clone(),
            SHOTGUN.clone(),
            CLAYMORE.clone(),
            FLAMETHROWER.clone(),
        ],
        mesh: Cone {
            sectors: 4,
            segments: 1,
            capped: false,
            base_radius: 0.75,
            apex_radius: 0.0,
        }
        .build(),
        exhaust: Emitter {
            emit_spread: polar(0.1, degs(15.0)),
            ..Emitter::default()
        },
        ..Ship::default()
    };
    let rock = Rock {
        pos: pt2(-400.0, -400.0),
        vel: vec2(2.0, 1.0),
        mesh: Icosphere(1.0, 1).build(),
        ..Rock::default()
    };
    let mut level = Level {
        player,
        rock,
        bounds: [pt2(-400.0, -600.0), pt2(400.0, 600.0)],
        camera: Camera::new(DIMS)
            .viewport((0..DIMS.0, DIMS.1..0))
            .transform(FollowPlayer::default())
            .perspective(Diagonal(degs(90.0)), 0.1..1000.0),
    };

    let mut paused = false;
    let start = Instant::now();
    let stats = win.run(|frame| {
        let w = &frame.win.imp;
        let _t = frame.t.as_secs_f32();
        let dt = frame.dt.as_secs_f32();

        let plr = &mut level.player;

        plr.acc = Vec2::zero();
        let mut target_rot = Angle::zero();
        if w.is_key_down(Key::Up) {
            plr.thrust();
        }
        if w.is_key_down(Key::Left) {
            target_rot = -Ship::ROT_RATE;
        }
        if w.is_key_down(Key::Right) {
            target_rot = Ship::ROT_RATE;
        }
        if w.is_key_down(Key::Space) {
            plr.fire();
        }
        if w.is_key_pressed(Key::P, KeyRepeat::No) {
            paused = !paused;
        }
        if w.is_key_pressed(Key::LeftAlt, KeyRepeat::No) {
            plr.guns.rotate_left(1);
        }

        let mut ctx = Context {
            camera: level.camera,
            framebuf: frame.buf,
            render_ctx: frame.ctx,
        };

        if !paused {
            plr.rotate(target_rot, dt);

            level.update(dt, &ctx);
        }

        // RENDER

        level.render(&mut ctx);
        Continue(())
    });

    let elapsed = start.elapsed();

    eprintln!(
        "time elapsed: {:.2?}, frames: {}, fps: {:.2}",
        elapsed,
        stats.frames,
        stats.frames / elapsed.as_secs_f32()
    );
}
