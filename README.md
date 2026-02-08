# GALAXY - Space Strategy Game

A real-time space strategy game inspired by GalaxyNG, built with Rust and Bevy.

## Running the Game

```bash
cargo run --bin galaxy
```

## Controls

- **SPACE** - Advance one turn (AI races make decisions)
- **Arrow Keys** - Pan camera around the galaxy
- **Mouse Wheel** - Zoom in/out
- **ESC** - Exit game

## Game Features

- **4 AI Races** competing for galactic dominance
- **Racebot AI** with different personalities (Aggressive, Defensive, Balanced, Expansionist)
- **Planet Production** - Resources, industry, population growth
- **Ship Building** - Design and build fleets
- **Cargo System** - Transport colonists, materials, and capital
- **Probabilistic Combat** - GalaxyNG-style combat resolution
- **Planet Bombing** - Conquer enemy worlds
- **Technology Advancement** - Drive, Weapons, Shields, Cargo

## Development

```bash
cargo fmt                                        # Format code
cargo test --workspace                           # Run tests
cargo clippy --workspace --all-targets -- -D warnings  # Lint
```

See `AGENTS.md` for detailed development guidelines.
