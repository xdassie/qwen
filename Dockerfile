FROM rust:1.95-bookworm

# Avoid prompts
ENV DEBIAN_FRONTEND=noninteractive

# Enable 32-bit architecture
RUN dpkg --add-architecture i386

# Install everything in one layer (cleaner + avoids cache issues)
RUN apt-get update && apt-get install -y \
    apt-transport-https \
    ca-certificates \
    build-essential \
    gcc-multilib \
    pkg-config \
    curl \
    git \
    # 32-bit core libs
    libc6-dev:i386 \
    libssl-dev:i386 \
    zlib1g-dev:i386 \
    # X11 / windowing
    libx11-dev:i386 \
    libxrandr-dev:i386 \
    libxi-dev:i386 \
    libxcursor-dev:i386 \
    libxinerama-dev:i386 \
    libwayland-dev:i386 \
    libxkbcommon-dev:i386 \
    # Audio / input
    libasound2-dev:i386 \
    libudev-dev:i386 \
    # Graphics stack
    libcairo2-dev:i386 \
    libpango1.0-dev:i386 \
    libglib2.0-dev:i386 \
    libgdk-pixbuf-2.0-dev:i386 \
    libfontconfig1-dev:i386 \
    libfreetype6-dev:i386 \
    libpng-dev:i386 \
    # 🔴 IMPORTANT: missing deps for pango/cairo
    libharfbuzz-dev:i386 \
    libfribidi-dev:i386 \
    && rm -rf /var/lib/apt/lists/*

# Add Rust target
RUN rustup target add i686-unknown-linux-gnu

# pkg-config setup for cross-compiling to 32-bit
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_LIBDIR=/usr/lib/i386-linux-gnu/pkgconfig
ENV PKG_CONFIG_PATH=/usr/lib/i386-linux-gnu/pkgconfig:/usr/share/pkgconfig

WORKDIR /app

COPY Cargo.toml .
COPY src ./src

#RUN cargo build --release --target i686-unknown-linux-gnu