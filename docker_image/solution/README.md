# Filler Rust Solution

## Building

**IMPORTANT**: You must compile the Rust solution **inside the Docker container** because the binary needs to be a Linux executable, not a Windows executable.

### Steps:

1. Build the Docker image (from the `docker_image` directory):
   ```bash
   docker build -t filler .
   ```

2. Run the container with the solution directory mounted:
   ```bash
   docker run -v "$(pwd)/solution":/filler/solution -it filler
   ```

3. **Inside the container**, compile the Rust solution:
   ```bash
   cd solution
   cargo build --release
   cd ..
   ```

The binary will be located at `solution/target/release/filler` (Linux binary).

## Running

After building inside the container, you can run the game:

```bash
./linux_game_engine -f maps/map01 -p1 solution/target/release/filler -p2 linux_robots/bender
```

Or test as player 2:

```bash
./linux_game_engine -f maps/map01 -p1 linux_robots/bender -p2 solution/target/release/filler
```

## How it works

The Rust implementation:
1. Reads input from stdin line by line
2. Parses the player number from the first line (`$$$ exec p<number>`)
3. Parses the Anfield (game board) when it receives the "Anfield" section
4. Parses the piece when it receives the "Piece" section
5. Finds a valid placement position (exactly one overlap with player's territory)
6. Outputs coordinates in the format `X Y\n`
7. Returns `0 0\n` if no valid placement is found

