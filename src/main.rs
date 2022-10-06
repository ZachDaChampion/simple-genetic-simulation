mod entities;

use entities::{Agent, Consumable, Entity, Pellet, SimpleAgent};
use nannou::prelude::*;

fn main() {
  nannou::app(model).update(update).simple_window(view).run();
}

struct Model {
  tick: u64,
  pellets: Vec<Pellet>,
  agents: Vec<SimpleAgent>,
}

fn model(app: &App) -> Model {
  let mut m = Model {
    tick: 0,
    pellets: Vec::new(),
    agents: Vec::new(),
  };

  for _ in 0..200 {
    m.pellets.push(Pellet::new(&app.window_rect(), None));
  }

  for _ in 0..50 {
    m.agents.push(SimpleAgent::new(
      map_range(
        random::<f32>(),
        0.0,
        1.0,
        app.window_rect().left(),
        app.window_rect().right(),
      ) as f32,
      map_range(
        random::<f32>(),
        0.0,
        1.0,
        app.window_rect().bottom(),
        app.window_rect().top(),
      ) as f32,
      entities::Stats {
        hp: 255,
        food: 255,
        speed: 10.0,
        tick_hunger: 1,
        starving_pain: 1,
      },
      None,
      None,
    ));
  }

  m
}

fn update(app: &App, model: &mut Model, _update: Update) {
  if model.pellets.len() < 200 && random::<u8>() < 48 {
    model.pellets.push(Pellet::new(&app.window_rect(), None));
  }

  let mut new_agents = Vec::new();
  let mut agent_count = model.agents.len();
  let bounds = &app.window_rect();
  model.agents.retain_mut(|agent| {
    model.pellets.retain(|pellet| {
      let (x, y) = agent.x_y();
      x < bounds.left()
        || x > bounds.right()
        || y < bounds.bottom()
        || y > bounds.top()
        || agent.eval_consumable(pellet, model.tick)
    });

    let res = &agent.run_tick(model.tick, &app.window_rect());

    if agent_count < 75
      && (random::<u8>() < 2 || agent_count < 10)
      && (agent.can_reproduce() || agent_count < 3)
    {
      agent_count += 1;
      new_agents.push(agent.reproduce(&app.window_rect()));
      println!("Reproduced: {:?}", new_agents.last().unwrap().weights);
    }

    *res
  });
  model.agents.append(&mut new_agents);

  model.tick += 1;
}

fn view(app: &App, model: &Model, frame: Frame) {
  let draw = app.draw();
  draw.background().color(BLACK);

  for pellet in &model.pellets {
    pellet.draw(&draw);
  }

  for agent in &model.agents {
    agent.draw(&draw);
  }

  draw.to_frame(app, &frame).unwrap();
}
