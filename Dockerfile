FROM debian:bookworm AS builder

# System dependencies
RUN apt update && apt install -y curl build-essential \
    libwebkit2gtk-4.1-dev libgtk-3-dev libssl-dev libsoup2.4-dev \
    libappindicator3-dev libayatana-appindicator3-dev librsvg2-dev \
    wget unzip xdg-utils

# Phidgets dependencies
RUN curl -fsSL https://www.phidgets.com/downloads/setup_linux | bash - 
RUN apt install -y libphidget22
RUN ln -s /lib/aarch64-linux-gnu/libphidget22.so.0 /usr/lib/libphidget22.so

# Node.js setup
# Use bash for the shell and ensure that nvm works
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

#Set up a bash profile for the root user
ENV BASH_ENV /root/.bash_env
RUN touch "${BASH_ENV}"
RUN echo '. "${BASH_ENV}"' >> /root/.bashrc

# Install NVM and Node.js
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.2/install.sh | bash \
    && export NVM_DIR="$HOME/.nvm" \
    && [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh" \
    && echo node > .nvmrc \
    && nvm install --lts \
    && nvm use --lts \
    && ln -s "$NVM_DIR/versions/node/$(nvm version)/bin/npm" /usr/local/bin/npm \
    && ln -s "$NVM_DIR/versions/node/$(nvm version)/bin/node" /usr/local/bin/node

# Rust setup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:$PATH"


# Set working directory and copy the app's source code into the container
WORKDIR /app
COPY . .

# Install dependencies (assuming you use npm or yarn in your app)
RUN bash -c "source ${BASH_ENV} && npm install"

# Build the app
RUN bash -c "source ${BASH_ENV} && npm run tauri build"

FROM scratch AS export

# Copy the built binary from the builder stage
# Adjust the path to where Tauri outputs your binary
COPY --from=builder /app/src-tauri/target/release/bundle /output


# Instructions:
# 1) Build the image
#   $docker build -t tauri-app-builder -f Dockerfile .
# 2) Create a temporary container. Does not run.
#   $docker create --name temp-container tauri-app-builder /bin/true
# 3) Extract the binary to Host ( ./output being the destination folder you want)
#   $docker cp temp-container:/output ../output
# 4) Clean up
#   $docker rm temp-container