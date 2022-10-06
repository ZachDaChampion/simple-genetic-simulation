# Simple Genetic Simulation

I wanted to experiment with Rust, and so I decided to recreate a screensave I made years ago in Java.

It is poorly optimized and likes to not run on some computers.

Halfway through I decided to shove in a genetic algorithm because I've been meaning to play with those for a while. I did this poorly.

## Behavior

The environment starts with 70 agents and 200 consumables. Each agent has a small change to reproduce every tick, as long as its health and hunger meters are high enough. If the population dwindles too low, the remaining agents are forced to reproduce to keep the simulation going.

Consumables come in the following forms:
- Green: "food," increases hunger (this makes the agent less hungry because naming is hard)
- Yellow: "medicine," increases health meter
- Purple: "poison," decreases health every few ticks for a set amount of time
- Orange: temporarily slows down an agent
- Blue: temporarily speeds up an agent

Ideally, agents learn to eat food and avoid poison to survive. In practice, they do this sometimes.
