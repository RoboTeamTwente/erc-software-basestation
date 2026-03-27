# Target Ubuntu 22.04 (glibc 2.35) to match the deployment machine
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# ── System dependencies ────────────────────────────────────────────────────────
RUN apt-get update && apt-get install -y \
    # Build essentials
    build-essential curl wget file unzip \
    # Tauri 2 system deps
    libwebkit2gtk-4.1-dev \
    libjavascriptcoregtk-4.1-dev \
    libsoup-3.0-dev \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    # AppImage tooling
    libfuse2 \
    squashfs-tools \
    xdg-utils \
    # GStreamer (required by gstreamer crate)
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-plugins-base \
    gstreamer1.0-plugins-good \
    && rm -rf /var/lib/apt/lists/*

# ── Rust ───────────────────────────────────────────────────────────────────────
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

# ── Node.js (via nvm — change NODE_VERSION as needed) ─────────────────────────
ENV NODE_VERSION=20
RUN curl -fsSL https://deb.nodesource.com/setup_${NODE_VERSION}.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# ── Bun ───────────────────────────────────────────────────────────────────────
RUN curl -fsSL https://bun.sh/install | bash
ENV PATH="/root/.bun/bin:${PATH}"

# ── Tauri CLI ──────────────────────────────────────────────────────────────────
RUN cargo install tauri-cli --locked

WORKDIR /app

# ── Build entrypoint ───────────────────────────────────────────────────────────
# These tell linuxdeploy NOT to bundle glib/gio/gvfs, which conflict with the host
ENV APPIMAGE_EXTRACT_AND_RUN=1
ENV LINUXDEPLOY_PLUGIN_GTK_EXCLUDE_LIBS="libglib-2.0.so.0,libgio-2.0.so.0,libgobject-2.0.so.0,libgmodule-2.0.so.0,libgthread-2.0.so.0"

# Mount your project at /app and run:  docker run --rm -v $(pwd):/app <image>
CMD ["cargo", "tauri", "build"]