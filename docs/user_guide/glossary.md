# Glossary

## Biomechanics

### SFB (Same Finger Bigram)

A sequence of two keys pressed by the same finger. This is the most expensive movement in typing, as the finger must travel from one key to another without the help of other fingers.

- *Goal*: Minimize (Target < 1.0%).

### Scissor

A transition between adjacent fingers (e.g., Ring to Middle) where the fingers move to different rows (e.g., Top to Bottom). This causes strain on the tendons connecting the fingers.

- *Goal*: Minimize.

### Redirect (Reversal)

A sequence of three keys where the direction of flow changes (e.g., Ring -> Middle -> Ring). This breaks the natural "rolling" motion of the hand.

### Roll

A sequence of keys pressed in a single direction (e.g., Pinky -> Ring -> Middle). This is the fastest and most comfortable motion.

- *Goal*: Maximize.

### Hand Balance

The ratio of keystrokes performed by the left hand vs. the right hand.

- *Goal*: 50/50.

## System Terms

### Hive

The central server application that coordinates optimization jobs, stores layouts, and manages the asset library.

### Agent / Node

A compute worker that connects to the Hive to process optimization jobs. It uses Simulated Annealing and Genetic Algorithms.

### Corpus

A large dataset of text used to analyze letter frequency. KeyForge uses N-grams (1-grams, 2-grams, 3-grams) and word lists from these corpora.

### Cost Matrix

A lookup table defining the "cost" (time/effort) to move between any two physical keys. This can be generic or personalized via the Typing Arena.

### Layer

A logical mapping of keys. Most modern keyboards use layers to access symbols, numbers, or navigation keys without moving hands from the home row.
