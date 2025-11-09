use minifb::{Key, Scale, WindowOptions};
use std::ops::ControlFlow::Continue;
use std::time::Instant;

use re::prelude::*;

use re::core::util::Dims;
use re::geom::solids::Build;
use re::prelude::color::gray;
use re::prelude::mat::ProjMat3;
use re_front::minifb::Window;

use entity::*;

mod entity;

const DIMS: Dims = (640, 360);

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

    let mut player = Ship::default();
    player.dir = vec2(0.0, -1.0);
    player.pos = pt2(DIMS.0 as f32 / 2.0, DIMS.1 as f32 / 2.0);

    let mut level = Level { player };

    let mut cooldown = 0.0;
    let start = Instant::now();
    win.run(|frame| {
        let w = &frame.win.imp;
        let t = frame.t.as_secs_f32();
        let dt = frame.dt.as_secs_f32();

        let plr = &mut level.player;

        plr.acc = Vec2::zero();
        plr.rot = Angle::zero();
        if w.is_key_down(Key::W) {
            plr.acc += 100.0 * plr.dir;
        }
        if w.is_key_down(Key::A) {
            plr.rot = turns(1.0);
        }
        if w.is_key_down(Key::D) {
            plr.rot = turns(-1.0);
        }

        if w.is_key_down(Key::Space) {
            plr.fire();
        }
        cooldown -= dt;
        plr.acc += vec2(0.0, 20.0); // Gravity

        level.update(dt);

        // RENDER

        let buf = &mut frame.buf.color_buf.buf;

        level.render(buf);

        let rock = re::geom::solids::Dodecahedron.build();

        let rot = rotate_y(turns(0.1 * t)).then(&rotate_z(turns(0.05 * t)));
        let mvp = scale(splat(50.0))
            .then(&rot)
            .then(&translate3(
                20.0 * t % DIMS.0 as f32,
                5.0 * t % DIMS.1 as f32,
                0.0,
            ))
            .to()
            .then(&orthographic(
                pt3(0.0, 0.0, -1e3),
                pt3(DIMS.0 as f32, DIMS.1 as f32, 1e3),
            ));

        Batch::new()
            .mesh(&rock)
            .shader(shader::new(
                |v: Vertex3<Normal3>, (mvp, n): (&ProjMat3<Model>, &Mat4)| {
                    vertex(mvp.apply(&v.pos), n.apply(&v.attrib).normalize())
                },
                |f: Frag<Normal3>| gray(f.var.x().abs()).to_color4(),
            ))
            .uniform((&mvp, &rot))
            .viewport(viewport(pt2(0, 0)..pt2(DIMS.0, DIMS.1)))
            .target(&mut frame.buf)
            .render();

        Continue(())
    });
    let stats = win.ctx.stats.borrow();
    let elapsed = start.elapsed();
    eprintln!(
        "time elapsed: {:.2?}, frames: {}, fps: {:.2}",
        elapsed,
        stats.frames,
        stats.frames / elapsed.as_secs_f32()
    );
}
