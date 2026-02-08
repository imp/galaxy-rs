# GALAXY - game simulator

## Introduction
GALAXY simulates a space populated by various planets of different size and characteristics.
The simulation revolves around development of the few races populating some of these planets.
Each race starts with a one single planet (all other planets are initially unexplored).
All the initially populated planets are of equal size (100 units). All other planets vary in size between 10 units and 300 units.
The simulation starts with a predefined number of races, randomly spread across the whole galaxy.
Each race can build ships to explore the galaxy. Each ship contains a space drive, a number of phase cannons and a weapon shield.

## The game space
The simulated game space is a 2 dimensional space. For the purpose of this simulation the size of the planet is negligible compared to the size of the galaxy.

## Gameplay
The gameplay revolves around the development of the races and their exploration of the galaxy.
Each race can build ships to explore the galaxy. Each ship contains a space drive, a number of phase cannons and a weapon shield.
The space drive allows the ship to travel between planets.
The phase cannons allow the ship to attack other ships or planets.
The weapon shield protects the ship from attacks.

As simulation progresses each races develops its drive technology, its weapon technology and its shield technology.
The development of the drive technology allows the race to build faster ships.
The development of the weapon technology allows the race to build more powerful weapons.
The development of the shield technology allows the race to build more powerful shields.

Each turn each planet can advance the technology of the race that owns it.
the amount of effort required to advance the technology is proportional to the size of the planet.
each race assigns which technology to advance on a per planet basis.

### Space ship construction
Each turn each planet produces some amount of material proportionate to its size.
The materials can be used to build ships. The amount of materials required to produce a single ship is calculated by its design.
Space ship design consists of three components: the hull, the engine, and the weapons.
Engine materials are required to build the engine. The more powerful the engine, the faster the ship can travel.
Hull materials are required to build the hull. The more powerful the hull, the more damage it can absorb.
Weapon materials are required to build the phase cannons. Each ship can have various number of cannons (zero or more).
The more powerful the cannon, the more damage it can inflict on an enemy ship when encountered.

### Space exploration
Races can order the ship they build on their planets to travel to other planets.
Whenever traveling ship arrives on uninhabited planet, the race is colonizing the planet.
When the ship arrives on a planet that is already inhabited, the relationship between the races determines what happens next.
Space ships can only travel between different planets and cannot target an arbitrary point in space.

### Races relationships
Relationship of a given race to any other race can be one of the three following:
- Friendly: The race is considered an ally and their ship will not be attacked when encountered.
- Hostile: The race is considered an enemy and their ship will be attacked when encountered.
- Neutral: The races are neither allies nor enemies and their ships will not be attacked when encountered.

In any case if the ships of a given race are being attacked by the ships of another race, the relationship with that race automatically becomes hostile and the attacked ships are attacking back.

## Game space initialization
The game is initialized with the size of the game space, number of planets, and number of races.

## Victory conditions
The game is won by the race that has the most planets at the end of the game.

# Implementation Details

The implementation uses `bevy` as the game engine.
