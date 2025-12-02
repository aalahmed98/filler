#!/bin/bash
# Test script for Filler bot
# Tests against all bots with 5 games each, displays final tally

# Enable exit on error for setup, but we'll disable it for game runs
set -e

echo "=========================================="
echo "  FILLER BOT TEST SUITE"
echo "=========================================="
echo ""

# Build Docker image if needed
echo "[1/3] Building Docker image..."
if docker build -t filler .; then
echo "  ✓ Docker image ready"
else
  echo "  ✗ Docker build failed!"
  exit 1
fi
echo ""

# Build the Rust solution
echo "[2/3] Building Rust solution..."
# Get absolute path - handle WSL/Git Bash path conversion for Windows Docker
CURRENT_DIR=$(pwd)
if [[ "$CURRENT_DIR" =~ ^/mnt/([a-z]) ]]; then
  # WSL path: /mnt/c/Users/... stays as /mnt/c/Users/...
  SOLUTION_PATH="$CURRENT_DIR/solution"
elif [[ "$CURRENT_DIR" =~ ^/([a-z]) ]]; then
  # Git Bash path: /c/Users/... -> //c/Users/... for Docker
  SOLUTION_PATH=$(echo "$CURRENT_DIR/solution" | sed 's|^/\([a-z]\)/|//\1/|')
else
  SOLUTION_PATH="$CURRENT_DIR/solution"
fi
docker run --rm -v "${SOLUTION_PATH}:/filler/solution" filler bash -c \
  "cd /filler/solution && cargo build --release > /dev/null 2>&1"
echo "  ✓ Build successful"
echo ""

# Run tests one by one and display results
echo "[3/3] Running tests..."
echo ""
  
  BOTS=("bender" "h2_d2" "wall_e" "terminator")
  MAPS=("map00" "map01" "map02")
  
declare -A results

# Initialize results
for bot in "${BOTS[@]}"; do
  for map in "${MAPS[@]}"; do
    results["${bot}_${map}"]=0
  done
done

game_count=0
total_games=$(( ${#BOTS[@]} * ${#MAPS[@]} * 5 ))

# Function to check if our bot (P1) won
check_win() {
  local output="$1"
  
  if echo "$output" | grep -qiE "Player1.*won|P1.*won"; then
    return 0
  fi
  
  if echo "$output" | grep -q "== O fin:"; then
    o_score=$(echo "$output" | grep "== O fin:" | sed "s/.*== O fin: \\([0-9]*\\).*/\\1/")
    x_score=$(echo "$output" | grep "== X fin:" | sed "s/.*== X fin: \\([0-9]*\\).*/\\1/")
    
    # P1 is O, P2 is X - we win if O score > X score
    if [ "$o_score" -gt "$x_score" ] 2>/dev/null; then
      return 0
    fi
  fi
  
  return 1
}

# Start a single container that we'll reuse for all games
echo "Starting test container..."
# Reuse the already computed SOLUTION_PATH
CONTAINER_ID=$(docker run -d -v "${SOLUTION_PATH}:/filler/solution" filler sh -c "tail -f /dev/null")
echo "  Container started: ${CONTAINER_ID:0:12}"

# Cleanup function
cleanup() {
  if [ -n "$CONTAINER_ID" ]; then
    echo ""
    echo "Cleaning up container..."
    docker stop "$CONTAINER_ID" > /dev/null 2>&1
    docker rm "$CONTAINER_ID" > /dev/null 2>&1
  fi
}

# Set trap to cleanup on exit
trap cleanup EXIT INT TERM
echo ""

# Function to create a modified map with new starting positions
# This runs inside the container to avoid path issues
create_modified_map() {
  local original_map=$1
  local game_num=$2
  local modified_map="${original_map}_modified_${game_num}_$$"
  
  # Create modified map inside container
  docker exec "$CONTAINER_ID" bash -c "
    map_file=\"/filler/maps/$original_map\"
    output_file=\"/filler/maps/$modified_map\"
    
    # Get map dimensions
    rows=\$(wc -l < \"\$map_file\" | tr -d ' ')
    cols=\$(head -1 \"\$map_file\" | wc -c | tr -d ' ')
    cols=\$((cols - 1))  # Subtract newline
    
    # Use game_num to generate varied but FAIR starting positions
    # Players should start on opposite sides of the map for fairness
    map_hash=\$(echo -n '$original_map' | od -An -N4 -tu4 | tr -d ' ')
    seed=\$(( $game_num * 1000 + map_hash ))
    
    # Calculate center of map
    center_row=\$((rows / 2))
    center_col=\$((cols / 2))
    
    # Generate an offset from center (varies by game)
    # Use different quadrants/positions based on game number
    case \$(( $game_num % 4 )) in
      0) # P1 top-left, P2 bottom-right
        p1_row=\$((rows / 4 + (seed % (rows / 4))))
        p1_col=\$((cols / 4 + (seed % (cols / 4))))
        p2_row=\$((3 * rows / 4 - (seed % (rows / 4))))
        p2_col=\$((3 * cols / 4 - (seed % (cols / 4))))
        ;;
      1) # P1 top-right, P2 bottom-left
        p1_row=\$((rows / 4 + (seed % (rows / 4))))
        p1_col=\$((3 * cols / 4 - (seed % (cols / 4))))
        p2_row=\$((3 * rows / 4 - (seed % (rows / 4))))
        p2_col=\$((cols / 4 + (seed % (cols / 4))))
        ;;
      2) # P1 bottom-left, P2 top-right
        p1_row=\$((3 * rows / 4 - (seed % (rows / 4))))
        p1_col=\$((cols / 4 + (seed % (cols / 4))))
        p2_row=\$((rows / 4 + (seed % (rows / 4))))
        p2_col=\$((3 * cols / 4 - (seed % (cols / 4))))
        ;;
      3) # P1 bottom-right, P2 top-left
        p1_row=\$((3 * rows / 4 - (seed % (rows / 4))))
        p1_col=\$((3 * cols / 4 - (seed % (cols / 4))))
        p2_row=\$((rows / 4 + (seed % (rows / 4))))
        p2_col=\$((cols / 4 + (seed % (cols / 4))))
        ;;
    esac
    
    # Ensure positions are within bounds
    p1_row=\$((p1_row < 0 ? 0 : p1_row))
    p1_col=\$((p1_col < 0 ? 0 : p1_col))
    p2_row=\$((p2_row < 0 ? 0 : p2_row))
    p2_col=\$((p2_col < 0 ? 0 : p2_col))
    p1_row=\$((p1_row >= rows ? rows - 1 : p1_row))
    p1_col=\$((p1_col >= cols ? cols - 1 : p1_col))
    p2_row=\$((p2_row >= rows ? rows - 1 : p2_row))
    p2_col=\$((p2_col >= cols ? cols - 1 : p2_col))
    
    # Use Python to modify the map - write script to temp file to avoid quoting issues
    python_script=\"/tmp/modify_map_\$\$.py\"
    cat > \"\$python_script\" << 'PYSCRIPT'
import sys
map_file = sys.argv[1]
output_file = sys.argv[2]
p1_row = int(sys.argv[3])
p1_col = int(sys.argv[4])
p2_row = int(sys.argv[5])
p2_col = int(sys.argv[6])

with open(map_file, 'r') as f:
    lines = [line.rstrip('\n\r') for line in f.readlines()]

# Replace all player markers with dots
for i in range(len(lines)):
    lines[i] = lines[i].replace('@', '.').replace('\$', '.')

# Place P1 (@) and P2 (\$)
lines[p1_row] = lines[p1_row][:p1_col] + '@' + lines[p1_row][p1_col+1:]
lines[p2_row] = lines[p2_row][:p2_col] + '\$' + lines[p2_row][p2_col+1:]

# Write modified map
with open(output_file, 'w') as f:
    for line in lines:
        f.write(line + '\n')
PYSCRIPT
    
    python3 \"\$python_script\" \"\$map_file\" \"\$output_file\" \"\$p1_row\" \"\$p1_col\" \"\$p2_row\" \"\$p2_col\"
    rm -f \"\$python_script\"
    
    # Verify file was created and has both markers
    if [ ! -f \"\$output_file\" ]; then
      echo \"ERROR: Failed to create modified map\" >&2
      exit 1
    fi
    
    # Verify both markers exist
    if ! grep -q '@' \"\$output_file\" || ! grep -q '\$' \"\$output_file\"; then
      echo \"ERROR: Modified map missing player markers\" >&2
      exit 1
    fi
    
    echo '$modified_map'
  " 2>&1
}

# Function to run a game and return output
run_game() {
  local bot=$1
  local map=$2
  local game_num=$3
  local output_file=$(mktemp)
  local exit_code
  
  # Create modified map with new starting positions
  local modified_map=$(create_modified_map "$map" "$game_num" 2>&1)
  
  # Check if map creation failed
  if [ -z "$modified_map" ] || echo "$modified_map" | grep -q "ERROR"; then
    echo "Failed to create modified map" > "$output_file"
    return 1
  fi
  
  # Use timeout to prevent hanging (5 minutes should be enough for any game)
  # Our bot is always P1, enemy bot is always P2
  timeout 300 docker exec "$CONTAINER_ID" bash -c \
    "cd /filler && ./linux_game_engine -f maps/$modified_map -p1 solution/target/release/filler -p2 linux_robots/$bot 2>&1" > "$output_file" 2>&1
  exit_code=$?
  
  # Clean up modified map
  docker exec "$CONTAINER_ID" rm -f "/filler/maps/$modified_map" 2>/dev/null
  
  cat "$output_file"
  rm -f "$output_file"
  return $exit_code
}

# Disable exit on error for game runs - we want to continue even if games fail
set +e

# Run games one by one
for bot in "${BOTS[@]}"; do
  echo "Testing vs ${bot^^}:"
  for map in "${MAPS[@]}"; do
    echo "  Map: ${map}"
    
    # Test our bot (P1) vs enemy bot (P2)
    wins=0
    for i in {1..5}; do
      ((game_count++))
      echo -n "    [${game_count}/${total_games}] Our bot (P1) vs ${bot} (P2) on ${map} (game ${i}/5)... "
      
      output=$(run_game "$bot" "$map" "$i")
      game_exit=$?
      
      if [ $game_exit -eq 124 ]; then
        echo "✗ TIMEOUT"
        continue
      elif [ $game_exit -ne 0 ]; then
        echo "✗ ERROR (exit code: $game_exit)"
        continue
      fi
      
      if check_win "$output"; then
        ((wins++))
        echo "✓ WIN"
      else
        echo "✗ LOSS"
      fi
    done
    results["${bot}_${map}"]=$wins
    echo "    Wins: ${wins}/5"
    echo ""
  done
  echo ""
done

# Re-enable exit on error for final display
set -e

# Cleanup will happen automatically via trap

# Display final summary
echo ""
echo "=========================================="
echo "           FINAL TALLY"
echo "=========================================="
echo ""

# Display results by bot
for bot in "${BOTS[@]}"; do
  echo "vs ${bot^^}:"
  total_wins=0
  for map in "${MAPS[@]}"; do
    wins=${results["${bot}_${map}"]:-0}
    total_wins=$((total_wins + wins))
    echo "  ${map} - ${wins}/5 wins"
  done
  echo "  TOTAL - ${total_wins}/15 wins"
  echo ""
done

echo "=========================================="
