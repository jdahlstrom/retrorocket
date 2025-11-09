use re::prelude::raster::line;
use re::prelude::*;

use crate::DIMS;

pub trait Entity {
    fn update(&mut self, dt: f32);
    fn render(&self, buf: &mut MutSlice2<u32>);
}

pub struct Level {
    pub player: Ship,
}

#[derive(Clone, Debug, Default)]
pub struct Ship {
    pub pos: Point2,
    pub dir: Vec2,
    pub vel: Vec2,

    pub acc: Vec2,
    pub rot: Angle,

    pub gun: Gun,
}

#[derive(Clone, Debug, Default)]
pub struct Gun {
    pub bullets: Vec<Bullet>,
    pub cooldown: f32,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Bullet {
    pub pos: Point2,
    pub vel: Vec2,
    pub life: f32,
}

impl Ship {
    pub fn fire(&mut self) {
        if self.gun.cooldown <= 0.0 {
            self.gun.bullets.push(Bullet {
                pos: self.pos + 10.0 * self.dir,
                vel: self.vel + 200.0 * self.dir,
                life: 1.0,
            });
            self.gun.cooldown = 0.1;
        }
    }
}

impl Entity for Level {
    fn update(&mut self, dt: f32) {
        self.player.update(dt);
    }

    fn render(&self, buf: &mut MutSlice2<u32>) {
        self.player.render(buf);
    }
}

impl Entity for Ship {
    fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.vel += self.acc * dt;

        self.dir = rotate2(self.rot * dt).apply(&self.dir);

        let k = 0.25;

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

        self.gun.cooldown -= dt;
        self.gun.bullets.update(dt);
    }

    fn render(&self, buf: &mut MutSlice2<u32>) {
        let p = self.pos;
        let d = 10.0 * self.dir;
        let a = (p + d).to_pt3().to();
        let b = (p + 0.3 * d.perp()).to_pt3().to();
        let c = (p - 0.3 * d.perp()).to_pt3().to();
        line([vertex(a, ()), vertex(b, ())], |sl| {
            buf[sl.y][sl.xs].fill(0xFF_FF_FF)
        });
        line([vertex(a, ()), vertex(c, ())], |sl| {
            buf[sl.y][sl.xs].fill(0xFF_FF_FF)
        });
        line([vertex(b, ()), vertex(c, ())], |sl| {
            buf[sl.y][sl.xs].fill(0xFF_FF_FF)
        });

        self.gun.bullets.render(buf);
    }
}

impl Entity for Vec<Bullet> {
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

    fn render(&self, buf: &mut MutSlice2<u32>) {
        for bul in self {
            let mut color = rgb(1.0, 1.0, 0.0);
            if bul.life < 0.2 {
                color *= inv_lerp(bul.life, 0.0, 0.2);
            }
            let [r, g, b] = color.to_color3().0;
            buf[[bul.pos.x() as _, bul.pos.y() as _]] = u32::from_be_bytes([0, r, g, b]);
        }
    }
}
