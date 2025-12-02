FROM rust:latest

# Set working directory
WORKDIR /filler

# Copy game engine, robots, and maps
COPY linux_game_engine /filler/linux_game_engine
COPY linux_robots /filler/linux_robots
COPY maps /filler/maps

# Make game engine executable
RUN chmod +x /filler/linux_game_engine

# Make robot executables
RUN chmod +x /filler/linux_robots/*

# The solution directory will be mounted as a volume at runtime
# No need to copy it here

# Default command
CMD ["/bin/bash"]

