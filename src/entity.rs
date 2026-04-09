use core::cell::RefCell;
use re::core::geom::Ray;
use re::prelude::*;
use std::ops::Sub;

use re::core::mat;
use re::core::math::color::gray;
use re::core::math::rand::{DefaultRng, Distrib, DEFAULT_RNG};
use re::core::render::cam::Transform;
use re::core::render::{shader, Context as RenderCtx, Model, View, World};
use re::front::minifb::Framebuf;
use re::geom::{solids, solids::Build};

use crate::vertex_shader;

pub trait Entity {
    fn update(&mut self, dt: f32, ctx: &Context);
    fn render(&self, ctx: &Context);
}

pub struct Level {
    pub bounds: [Point2<World>; 2],
    pub camera: Camera<FollowPlayer>,
    pub player: Ship,
    pub rock: Rock,
}

pub struct Context<'a, 'fb> {
    pub camera: Camera<FollowPlayer>,
    pub framebuf: &'a RefCell<Framebuf<'fb>>,
    pub render_ctx: &'a RenderCtx,
}

impl<'a, 'fb: 'a> Context<'a, 'fb> {
    pub fn batch(&self) -> Batch<(), (), (), (), &'a RefCell<Framebuf<'fb>>, &RenderCtx> {
        Batch::new()
            .viewport(self.camera.viewport)
            .target(self.framebuf)
            .context(&self.render_ctx)
    }
}

#[derive(Copy, Clone)]
pub struct FollowPlayer {
    pub pos: Point2<World>,
    pub target_pos: Point2<World>,
    pub distance: f32,
}

#[derive(Clone, Debug, Default)]
pub struct Ship {
    pub pos: Point2<World>,
    pub dir: Vec2<World>,
    pub vel: Vec2<World>,

    pub acc: Vec2<World>,
    pub rot: Angle,

    pub guns: Vec<Gun>,

    pub exhaust: Emitter,

    pub mesh: Mesh<Normal3>,

    pub rng: DefaultRng,
}

#[derive(Clone, Debug, Default)]
pub struct Rock {
    pub pos: Point2,
    pub dir: SphericalVec,
    pub vel: Vec2,
    pub mesh: Mesh<Normal3>,
}

#[derive(Clone, Debug, Default)]
pub struct Gun {
    pub muzzle_vel: f32,
    pub spread: PolarVec,
    pub lifetime: f32,
    pub burst: Burst,

    pub cooldown: f32,
    pub bullets: Emitter,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Burst {
    pub shots: u32,
    pub interval: f32,
    pub cooldown: f32,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Particle {
    pub pos: Point2<World>,
    pub vel: Vec2<World>,
    pub life: f32,
    pub fade: f32,
}

#[derive(Clone, Debug, Default)]
pub struct Emitter {
    pub pos: Point2<World>,
    pub vel: Vec2<World>,

    pub emit_vel: Vec2<World>,
    pub emit_spread: PolarVec,

    pub interval: f32, // secs per particle, 0 to emit all at once
    pub count: u32,    // number to emit in total
    pub time_to_next: f32,

    pub particles: Vec<Particle>,
    pub rng: DefaultRng,
}

// Inherent impls

impl Level {
    fn update_camera(&mut self) {
        let FollowPlayer {
            pos, target_pos, ..
        } = &mut self.camera.transform;
        let [mn, mx] = self.bounds;
        let pad = vec2(100.0, 40.0);
        *target_pos = (self.player.pos + 0.3 * self.player.vel).clamp(&(mn + pad), &(mx - pad));
        *pos = pos.lerp(target_pos, 0.1);
    }

    fn render_background(&self, ctx: &Context) {
        let mw = Mat4::identity();
        let mvp = mw.then(&self.camera.world_to_project());

        let left_bot_near = self.bounds[0].to_pt3().to() - 200.0 * Vec3::Z;
        let right_top_far = self.bounds[1].to_pt3().to() + 100.0 * Vec3::Z;
        ctx.batch()
            .mesh::<TexCoord>(
                &solids::Box {
                    left_bot_near,
                    right_top_far,
                }
                .build(),
            )
            .shader(shader::new(vertex_shader, |f: Frag<TexCoord>| {
                let u = (f.var.u() * 20.0) as u8;
                let v = (f.var.v() * 30.0) as u8;

                gray(((u & 1) ^ (v & 1)) * 0x33 + 0x33).to_rgba()
            }))
            .uniform((&mvp, &Mat3::identity()))
            .viewport(self.camera.viewport)
            .render();
    }
}

impl Ship {
    pub const LIN_ACC: f32 = 120.0;
    pub const ROT_ACC: f32 = 10.0;
    pub const ROT_RATE: Angle = turns(1.0);

    pub fn rotate(&mut self, target_rate: Angle, dt: f32) {
        self.rot = self.rot.lerp(&target_rate, Ship::ROT_ACC * dt);
    }

    pub fn fire(&mut self) {
        //let to_world = self.to_world3();

        let gun = &mut self.guns[0];

        if gun.cooldown > 0.0 {
            return;
        }

        gun.bullets.emit_n(gun.burst.shots, gun.burst.interval);
        gun.bullets.emit_spread = gun.spread;
        gun.cooldown = gun.burst.cooldown;
    }

    pub fn thrust(&mut self) {
        self.acc += Self::LIN_ACC * self.dir;
        self.exhaust.emit_n(10, 0.01);
    }

    pub fn _to_world3(&self) -> Mat3<Model, World> {
        let y = 5.0 * self.dir;
        let x = y.perp();
        mat![
            x.x(), y.x(), self.pos.x();
            x.y(), y.y(), self.pos.y();
            0.0, 0.0, 1.0
        ]
    }
    pub fn to_world4(&self) -> Mat4<Model, World> {
        let y = 5.0 * self.dir;
        let x = y.perp();
        mat![
            x.x(), y.x(), 0.0, self.pos.x();
            x.y(), y.y(), 0.0, self.pos.y();
            0.0, 0.0, 1.0, 0.0;
            0.0, 0.0, 0.0, 1.0;
        ]
    }
}

impl Gun {
    pub const fn _new() -> Self {
        Self {
            muzzle_vel: 0.0,
            spread: polar(0.0, degs(0.0)),
            lifetime: 0.0,
            burst: Burst {
                shots: 0,
                interval: 0.0,
                cooldown: 0.0,
            },
            cooldown: 0.0,
            bullets: Emitter::new(),
        }
    }

    pub fn _fire() {}
}

impl Emitter {
    pub const fn new() -> Self {
        Self {
            pos: pt2(0.0, 0.0),
            vel: vec2(0.0, 0.0),

            emit_vel: vec2(0.0, 0.0),
            emit_spread: polar(0.0, degs(0.0)),

            interval: 0.0,
            time_to_next: 0.0,
            count: 0,

            particles: vec![],
            rng: DEFAULT_RNG,
        }
    }

    pub fn emit_n(&mut self, count: u32, interval: f32) {
        self.count = count;
        self.interval = interval;
    }
}

// Trait impls

impl Default for FollowPlayer {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            target_pos: Default::default(),
            distance: 200.0,
        }
    }
}

impl Transform for FollowPlayer {
    fn world_to_view(&self) -> Mat4<World, View> {
        translate(-self.pos.to_pt3().to_vec().to() + self.distance * Vec3::Z).to()
    }
}

impl Entity for Level {
    fn update(&mut self, dt: f32, ctx: &Context) {
        self.player.acc += vec2(0.0, -10.0); // Gravity

        self.player.update(dt, ctx);

        self.rock.update(dt, ctx);

        let [mn, mx] = self.bounds;

        let p = &mut self.player;

        let ray = Ray(p.pos, p.dir);

        self.update_camera();
    }

    fn render(&self, ctx: &Context) {
        self.render_background(ctx);
        self.player.render(ctx);
        self.rock.render(ctx);
    }
}

impl Entity for Ship {
    fn update(&mut self, dt: f32, ctx: &Context) {
        self.pos += self.vel * dt;
        self.vel += self.acc * dt;

        self.dir = rotate2(self.rot * dt).to().apply(&self.dir).normalize();

        self.exhaust.pos = self.pos;
        self.exhaust.vel = self.vel;
        self.exhaust.emit_vel = -50.0 * self.dir;

        self.exhaust.update(dt, ctx);
        for gun in &mut self.guns {
            gun.bullets.pos = self.pos;
            gun.bullets.vel = self.vel;
            gun.bullets.emit_vel = gun.muzzle_vel * self.dir;
            gun.update(dt, ctx);
        }
    }

    fn render(&self, ctx: &Context) {
        self.exhaust.render(ctx);
        for gun in &self.guns {
            gun.render(ctx);
        }

        let mvp = self.to_world4().then(&ctx.camera.world_to_project());

        ctx.batch()
            .mesh(&self.mesh)
            .shader(shader::new(vertex_shader, |_f: Frag<Normal3>| {
                rgba(0xFF, 0x33, 0x33, 0xFF)
            }))
            .uniform((&mvp, &Mat3::identity()))
            .render();
    }
}

impl Entity for Gun {
    fn update(&mut self, dt: f32, ctx: &Context) {
        self.cooldown = self.cooldown.sub(dt).max(0.0);
        self.bullets.update(dt, ctx);
    }

    fn render(&self, ctx: &Context) {
        self.bullets.render(ctx);
    }
}

impl Entity for Rock {
    fn update(&mut self, dt: f32, ctx: &Context) {
        self.pos += self.vel * dt;

        self.dir = spherical(
            1.0,
            self.dir.az() + turns(0.1) * dt,
            self.dir.alt() + turns(0.12) * dt,
        );
    }

    fn render(&self, ctx: &Context) {
        let rot = rotate_y(self.dir.az()).then(&rotate_z(self.dir.alt()));
        let mvp = scale(splat(50.0))
            .then(&rot)
            .then(&translate3(self.pos.x(), self.pos.y(), 0.0))
            .to()
            .then(&ctx.camera.world_to_project());

        ctx.batch()
            .mesh(&self.mesh)
            .shader(shader::new(vertex_shader, |f: Frag<Normal3>| {
                gray((-f.var.z()).max(0.0)).to_color4()
            }))
            .uniform((&mvp, &rot))
            .render();
    }
}

impl Entity for Emitter {
    fn update(&mut self, dt: f32, _ctx: &Context) {
        let mut emit = || {
            let spread = self.emit_spread / 2.0;
            let spread = (-spread..spread).sample(&mut self.rng);
            let rot = rotate2(spread.az()).to();
            let emit_vel = rot.apply(&self.emit_vel) * (1.0 + spread.r());

            self.particles.push(Particle {
                pos: self.pos,
                vel: self.vel + emit_vel,
                life: 1.0,
                fade: 0.2,
            });
        };

        if self.count > 0 {
            if self.interval < dt {
                let mut dt = dt;
                while self.interval < dt && self.count > 0 {
                    emit();
                    dt -= self.interval;
                    self.count -= 1;
                }
            } else if self.time_to_next <= 0.0 {
                emit();
                self.count -= 1;
                self.time_to_next += self.interval;
            } else {
                self.time_to_next -= dt;
            }
        }

        let mut i = 0;
        loop {
            if i >= self.particles.len() {
                break;
            }
            let p = &mut self.particles[i];
            p.life -= dt;
            p.pos += dt * p.vel;
            if p.life <= 0.0 {
                self.particles.swap_remove(i);
            } else {
                i += 1
            }
        }
    }

    fn render(&self, ctx: &Context) {
        let buf = &mut ctx.framebuf.borrow_mut().color_buf.buf;
        for bul in &self.particles {
            let mut color = rgb(1.0, 1.0, 0.0);
            if bul.life < bul.fade {
                color *= bul.life / bul.fade;
            }
            let [r, g, b] = color.to_color3().0;

            let pos_wld = bul.pos.to_pt3();

            let pos_clip = ctx.camera.world_to_project().apply(&pos_wld);
            let [x, y, z, w] = pos_clip.0;
            if x.abs() > w.abs() || y.abs() > w.abs() || z.abs() > w.abs() {
                continue;
            }

            let ndc = pt3(x / w, y / w, z / w); // ndc
            let [x, y, _] = ctx.camera.viewport.apply(&ndc).0;

            buf[[x as _, y as _]] = u32::from_be_bytes([0, r, g, b]);
        }
    }
}
