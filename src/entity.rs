use re::prelude::*;

use crate::{vertex_shader, DIMS};
use re::core::{
    math::color::gray,
    math::rand::{DefaultRng, Distrib},
    render::raster::line,
};
use re::front::minifb::Framebuf;

pub trait Entity {
    fn update(&mut self, dt: f32);
    fn render(&self, buf: &mut Framebuf);
}

pub struct Level {
    pub player: Ship,
    pub rock: Rock,
}

#[derive(Clone, Debug, Default)]
pub struct Ship {
    pub pos: Point2,
    pub dir: Vec2,
    pub vel: Vec2,

    pub acc: Vec2,
    pub rot: Angle,

    pub guns: Vec<Gun>,

    pub thrust: bool,
    pub cooldown: f32,
    pub exhaust: Vec<Particle>,

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
    pub cooldown: f32,
    pub muzzle_vel: f32,
    pub spread: PolarVec,
    pub shots: u32,

    pub bullets: Vec<Particle>,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Particle {
    pub pos: Point2,
    pub vel: Vec2,
    pub life: f32,
    pub fade: f32,
}

pub struct Emitter {
    pub rate: f32,
    pub particles: Vec<Particle>,
}

// Inherent impls

impl Ship {
    pub const LIN_ACC: f32 = 120.0;
    pub const ROT_ACC: f32 = 20.0;
    pub const ROT_RATE: Angle = turns(1.0);

    pub fn rotate(&mut self, target_rate: Angle, dt: f32) {
        self.rot = self.rot.lerp(&target_rate, Ship::ROT_ACC * dt);
    }

    pub fn fire(&mut self) {
        let gun = &mut self.guns[0];
        if self.cooldown <= 0.0 {
            for _ in 0..gun.shots {
                let vel_spread = (0.0..gun.spread.r()).sample(&mut self.rng);
                let ang_spread =
                    (-gun.spread.az().to_rads()..gun.spread.az().to_rads()).sample(&mut self.rng);

                gun.bullets.push(Particle {
                    pos: self.pos + 10.0 * (self.dir),
                    vel: self.vel
                        + gun.muzzle_vel
                            * ((1.0 + vel_spread) * self.dir * ang_spread.cos()
                                + ang_spread.sin() * self.dir.perp()),
                    life: 2.0,
                    fade: 0.2,
                });
            }
            self.cooldown = gun.cooldown;
        }
    }

    pub fn thrust(&mut self) {
        self.acc += Self::LIN_ACC * self.dir;
        self.thrust = true;
    }
}

impl Gun {}

// Trait impls

impl Entity for Level {
    fn update(&mut self, dt: f32) {
        self.player.update(dt);
        self.rock.update(dt);
    }

    fn render(&self, buf: &mut Framebuf) {
        self.player.render(buf);
        self.rock.render(buf);
    }
}

impl Entity for Ship {
    fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.vel += self.acc * dt;

        self.dir = rotate2(self.rot * dt).apply(&self.dir);

        let k = 0.25;

        // TODO :E
        if self.pos.x() < 10.0 {
            self.pos[0] = 10.0;
            self.vel = k * vec2(-self.vel.x(), self.vel.y());
        }
        if self.pos.x() > DIMS.0 as f32 - 10.0 {
            self.pos[0] = DIMS.0 as f32 - 10.0;
            self.vel = k * vec2(-self.vel.x(), self.vel.y());
        }
        if self.pos.y() < 10.0 {
            self.pos[1] = 10.0;
            self.vel = k * vec2(self.vel.x(), -self.vel.y());
        }
        if self.pos.y() > DIMS.1 as f32 - 10.0 {
            self.pos[1] = DIMS.1 as f32 - 10.0;
            self.vel = k * vec2(self.vel.x(), -self.vel.y());
        }

        if self.thrust {
            for _ in 0..10 {
                let disp_x = (-0.2..0.2).sample(&mut self.rng);
                let disp_y = (0.0..0.2).sample(&mut self.rng);
                self.exhaust.push(Particle {
                    pos: self.pos,
                    vel: self.vel - 50.0 * ((1.0 + disp_y) * self.dir + disp_x * self.dir.perp()),
                    life: 0.4,
                    fade: 0.3,
                })
            }
        }

        self.cooldown -= dt;
        self.exhaust.update(dt);
        for gun in &mut self.guns {
            gun.bullets.update(dt);
        }
    }

    fn render(&self, buf: &mut Framebuf) {
        self.exhaust.render(buf);
        for gun in &self.guns {
            gun.bullets.render(buf);
        }

        let p = self.pos;
        let d = 10.0 * self.dir;
        let a = (p + d).to_pt3().to();
        let b = (p + 0.3 * d.perp()).to_pt3().to();
        let c = (p - 0.3 * d.perp()).to_pt3().to();

        let buf = &mut buf.color_buf.buf;
        line([vertex(a, ()), vertex(b, ())], |sl| {
            buf[sl.y][sl.xs].fill(0xFF_FF_FF)
        });
        line([vertex(a, ()), vertex(c, ())], |sl| {
            buf[sl.y][sl.xs].fill(0xFF_FF_FF)
        });
        line([vertex(b, ()), vertex(c, ())], |sl| {
            buf[sl.y][sl.xs].fill(0xFF_FF_FF)
        });
    }
}

impl Entity for Rock {
    fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;

        self.dir = spherical(
            1.0,
            self.dir.az() + turns(0.1) * dt,
            self.dir.alt() + turns(0.12) * dt,
        );
    }

    fn render(&self, buf: &mut Framebuf) {
        let rot = rotate_y(self.dir.az()).then(&rotate_z(self.dir.alt()));
        let mvp = scale(splat(50.0))
            .then(&rot)
            .then(&translate3(self.pos.x(), self.pos.y(), 0.0))
            .to()
            .then(&orthographic(
                pt3(0.0, 0.0, -1e3),
                pt3(DIMS.0 as f32, DIMS.1 as f32, 1e3),
            ));

        Batch::new()
            .mesh(&self.mesh)
            .shader(shader::new(vertex_shader, |f: Frag<Normal3>| {
                gray(f.var.z().max(0.0)).to_color4()
            }))
            .uniform((&mvp, &rot))
            .viewport(viewport(pt2(0, 0)..pt2(DIMS.0, DIMS.1)))
            .target(buf)
            .render();
    }
}

impl Entity for Vec<Particle> {
    fn update(&mut self, dt: f32) {
        for i in 0.. {
            if i >= self.len() {
                break;
            }
            let b = &mut self[i];
            b.life -= dt;
            b.pos += dt * b.vel;
            if b.pos.x() < 0.0
                || b.pos.y() < 0.0
                || b.pos.x() >= DIMS.0 as f32
                || b.pos.y() >= DIMS.1 as f32
            {
                b.life = 0.0;
            }
            if b.life <= 0.0 {
                self.swap_remove(i);
            }
        }
    }

    fn render(&self, buf: &mut Framebuf) {
        let buf = &mut buf.color_buf.buf;
        for bul in self {
            let mut color = rgb(1.0, 1.0, 0.0);
            if bul.life < bul.fade {
                color *= bul.life / bul.fade;
            }
            let [r, g, b] = color.to_color3().0;
            buf[[bul.pos.x() as _, bul.pos.y() as _]] = u32::from_be_bytes([0, r, g, b]);
        }
    }
}
