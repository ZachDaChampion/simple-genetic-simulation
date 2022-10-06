use nannou::prelude::*;

fn rand_range(min: f32, max: f32) -> f32 {
  let val = random::<f32>();
  let diff = max - min;
  min + (diff * val)
}

fn rand_weights() -> Weights {
  Weights {
    poison: rand_range(-10.0, 5.0),
    heal: rand_range(-5.0, 10.0),
    feed: rand_range(-8.0, 8.0),
    slow_down: rand_range(-5.0, 5.0),
    speed_up: rand_range(-5.0, 5.0),
  }
}

#[derive(Copy, Clone)]
pub struct InstantEffect {
  pub change: f32,
}

#[derive(Copy, Clone)]
pub struct IntervalEffect {
  pub change: f32,
  pub tick_interval: u8,
  pub duration: u16,
}

#[derive(Copy, Clone)]
pub struct DurationEffect {
  pub change: f32,
  pub duration: u16,
}

#[derive(Copy, Clone)]
pub enum Effect {
  Nothing,
  Poison(IntervalEffect),
  Heal(InstantEffect),
  Feed(InstantEffect),
  SpeedUp(DurationEffect),
  SlowDown(DurationEffect),
}

pub trait Entity {
  fn x_y(&self) -> (f32, f32);
  fn draw(&self, draw: &Draw);

  fn dist_sq(&self, other: &impl Entity) -> f64 {
    let (other_x, other_y) = other.x_y();
    let (x, y) = self.x_y();
    ((other_x - x) as f64).pow(2) + ((other_y - y) as f64).pow(2)
  }

  fn dist(&self, other: &impl Entity) -> f32 {
    self.dist_sq(other).sqrt() as f32
  }
}

pub trait Agent: Entity {
  fn new(
    x: f32,
    y: f32,
    stats: Stats,
    weights: Option<Weights>,
    color: Option<rgb::Srgb<u8>>,
  ) -> Self;
  fn eval_consumable(&mut self, other: &impl Consumable, tick: u64) -> bool;
  fn run_tick(&mut self, tick: u64, bounds: &geom::Rect<f32>) -> bool;
  fn can_reproduce(&self) -> bool;
  fn reproduce(&self, bounds: &geom::Rect<f32>) -> Self;
}

pub trait Consumable: Entity {
  fn new(bounds: &geom::Rect<f32>, effect: Option<Effect>) -> Self;
  fn radius(&self) -> f32;
  fn effect(&self) -> Effect;
}

pub struct Pellet {
  pub x: f32,
  pub y: f32,
  pub radius: f32,
  pub effect: Effect,
}

impl Entity for Pellet {
  fn x_y(&self) -> (f32, f32) {
    (self.x, self.y)
  }

  fn draw(&self, draw: &Draw) {
    let (x, y) = self.x_y();
    draw
      .ellipse()
      .color(match self.effect {
        Effect::Nothing => WHITE,
        Effect::Feed(_) => GREEN,
        Effect::Heal(_) => YELLOW,
        Effect::Poison(_) => PURPLE,
        Effect::SlowDown(_) => ORANGE,
        Effect::SpeedUp(_) => BLUE,
      })
      .radius(self.radius)
      .x_y(x, y);
  }
}

impl Consumable for Pellet {
  fn new(bounds: &geom::Rect<f32>, effect: Option<Effect>) -> Self {
    Pellet {
      x: map_range(random::<f32>(), 0.0, 1.0, bounds.left(), bounds.right()) as f32,
      y: map_range(random::<f32>(), 0.0, 1.0, bounds.bottom(), bounds.top()) as f32,
      radius: 3.0,
      effect: effect.unwrap_or(match random::<u8>() {
        0..=51 => Effect::Poison(IntervalEffect {
          change: -10.0,
          tick_interval: 3,
          duration: 60,
        }),
        52..=103 => Effect::Heal(InstantEffect { change: 25.0 }),
        104..=154 => Effect::Feed(InstantEffect { change: 64.0 }),
        155..=205 => Effect::SpeedUp(DurationEffect {
          change: 2.0,
          duration: 180,
        }),
        206..=255 => Effect::SlowDown(DurationEffect {
          change: -2.0,
          duration: 180,
        }),
      }),
    }
  }

  fn radius(&self) -> f32 {
    self.radius
  }

  fn effect(&self) -> Effect {
    self.effect
  }
}

#[derive(Debug)]
pub struct Weights {
  poison: f32,
  heal: f32,
  feed: f32,
  slow_down: f32,
  speed_up: f32,
}

#[derive(Copy, Clone)]
pub struct Stats {
  pub hp: i32,
  pub food: i32,
  pub speed: f32,
  pub tick_hunger: u16,
  pub starving_pain: u16,
}

pub struct SimpleAgent {
  x: f32,
  y: f32,
  xv: f32,
  yv: f32,
  hp: i32,
  food: i32,
  base_stats: Stats,
  stats: Stats,
  pub weights: Weights,
  effects: Vec<(Effect, u64)>,
  color: rgb::Srgb<u8>,
}

impl Entity for SimpleAgent {
  fn x_y(&self) -> (f32, f32) {
    (self.x, self.y)
  }

  fn draw(&self, draw: &Draw) {
    let angle = f32::atan2(self.yv, self.xv);
    draw
      .xy(vec2(self.x, self.y))
      .rotate(angle)
      .tri()
      .points((-4.0, 3.0), (-4.0, -3.0), (3.0, 0.0))
      .color(self.color);
  }
}

impl Agent for SimpleAgent {
  fn new(
    x: f32,
    y: f32,
    stats: Stats,
    weights: Option<Weights>,
    color: Option<rgb::Srgb<u8>>,
  ) -> Self {
    SimpleAgent {
      x: x,
      y: y,
      xv: 0.0,
      yv: 0.0,
      hp: stats.hp,
      food: stats.food,
      base_stats: stats,
      stats: stats,
      weights: weights.unwrap_or(rand_weights()),
      effects: Vec::new(),
      color: color.unwrap_or(WHEAT),
    }
  }

  fn eval_consumable(&mut self, other: &impl Consumable, tick: u64) -> bool {
    let effect_weight = match other.effect() {
      Effect::Poison(_) => self.weights.poison,
      Effect::Heal(_) => self.weights.heal,
      Effect::Feed(_) => self.weights.feed,
      Effect::SlowDown(_) => -self.weights.slow_down,
      Effect::SpeedUp(_) => self.weights.speed_up,
      Effect::Nothing => 0.0,
    };

    let dist_sq = self.dist_sq(other);
    let (other_x, other_y) = other.x_y();

    // Eat
    if dist_sq < other.radius().pow(2) as f64 + 36.0 {
      self.effects.push((other.effect(), tick));
      return false;
    }

    let xv_delta = effect_weight * (other_x - self.x) / dist_sq as f32;
    let yv_delta = effect_weight * (other_y - self.y) / dist_sq as f32;

    self.xv += xv_delta;
    self.yv += yv_delta;

    // self.xv += if xv_delta.abs() < self.stats.speed * 3.0 {
    //   xv_delta
    // } else {
    //   self.stats.speed * 3.0
    // };

    // self.yv += if yv_delta.abs() < self.stats.speed * 3.0 {
    //   yv_delta
    // } else {
    //   self.stats.speed * 3.0
    // };

    true
  }

  fn run_tick(&mut self, tick: u64, bounds: &geom::Rect<f32>) -> bool {
    if self.xv.is_infinite() || self.yv.is_infinite() {
      return false;
    }

    // Apply effects
    self.effects.retain(|effect| match effect {
      (Effect::Nothing, _) => false,
      (Effect::Poison(data), start_tick) => {
        if tick >= start_tick + data.duration as u64 {
          self.color = WHITE;
          false;
        }
        if (tick - start_tick) % data.tick_interval as u64 == 0 {
          self.hp = self.hp.saturating_add(data.change as i32);
        }
        self.color = PURPLE;
        true
      }
      (Effect::Heal(data), start_tick) => {
        if tick == *start_tick {
          self.hp = self.hp.saturating_add(data.change as i32);
        }
        false
      }
      (Effect::Feed(data), start_tick) => {
        if tick == *start_tick {
          self.food = self.food.saturating_add(data.change as i32);
        }
        false
      }
      (Effect::SpeedUp(data), start_tick) => {
        if tick == *start_tick {
          self.stats.speed += data.change;
        } else if tick >= start_tick + data.duration as u64 {
          self.stats.speed -= data.change;
          return false;
        }
        true
      }
      (Effect::SlowDown(data), start_tick) => {
        if tick == *start_tick {
          self.stats.speed += data.change;
        } else if tick >= start_tick + data.duration as u64 {
          self.stats.speed -= data.change;
          return false;
        }
        true
      }
    });

    // Check bounds
    if self.hp > self.stats.hp {
      self.hp = self.stats.hp;
    }
    if self.food > self.stats.food {
      self.food = self.stats.food;
    }
    let speed_sq: f64 = (self.xv as f64).pow(2) + (self.yv as f64).pow(2);
    let speed = speed_sq.sqrt() as f32;
    if speed.is_infinite() {
      return false;
    };
    if speed > self.stats.speed {
      self.xv *= self.stats.speed / speed;
      self.yv *= self.stats.speed / speed;
    }

    // Apply food
    self.food = self.food.saturating_sub(self.stats.tick_hunger as i32);
    if self.food <= 0 {
      self.hp = self.hp.saturating_sub(self.stats.starving_pain as i32);
    }

    // Check death
    if self.hp <= 0 {
      return false;
    }

    // Apply impulse
    self.x += self.xv;
    self.y += self.yv;

    if self.x < bounds.left()
      || self.x > bounds.right()
      || self.y < bounds.bottom()
      || self.y > bounds.top()
    {
      return false;
    }

    true
  }

  fn can_reproduce(&self) -> bool {
    let min_food = self.stats.food as f32 * 0.67;
    let min_hp = self.stats.hp as f32 * 0.33;
    self.food as f32 > min_food && self.hp as f32 > min_hp
    // true
  }

  fn reproduce(&self, bounds: &geom::Rect<f32>) -> Self {
    let weights = Weights {
      poison: self.weights.poison + random::<f32>() * 10.0 - 5.0,
      heal: self.weights.heal + random::<f32>() * 10.0 - 5.0,
      feed: self.weights.feed + random::<f32>() * 10.0 - 5.0,
      slow_down: self.weights.slow_down + random::<f32>() * 10.0 - 5.0,
      speed_up: self.weights.speed_up + random::<f32>() * 10.0 - 5.0,
    };
    Self::new(
      map_range(random::<u8>(), 0, 255, bounds.left(), bounds.right()),
      map_range(random::<u8>(), 0, 255, bounds.bottom(), bounds.top()),
      // self.x,
      // self.y,
      self.base_stats,
      Some(weights),
      Some(WHITE),
    )
  }
}
