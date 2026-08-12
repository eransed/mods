# syntax=docker/dockerfile:1
FROM ubuntu:24.04
RUN apt update && apt upgrade -y
RUN apt install git file tree curl pkg-config libssl-dev build-essential -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# RUN echo hello

# Use bash for the shell
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# Create a script file sourced by both interactive and non-interactive bash shells
# ENV BASH_ENV="${HOME}/.bash_env"
ENV BASH_ENV="/.bash_env"
RUN touch "${BASH_ENV}"
RUN echo '. "${BASH_ENV}"' >> ~/.bashrc

# Download and install nvm
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.5/install.sh | PROFILE="${BASH_ENV}" bash
RUN echo node > .nvmrc
RUN nvm install 24
# RUN node --version
RUN npm install -g quicktype

WORKDIR /build
COPY . .
RUN cargo build --release
# RUN ls -lha
# RUN ls -lha target
# RUN tree .
# RUN file target/release/mods

RUN mkdir -p /out
RUN cp target/release/mods /out
RUN cp build_info.json /out
# RUN ls -lh /out
# RUN cat build_info.json
# RUN ldd --version
# RUN ldd /out/mods # not a dynamic executable
RUN cargo test

CMD ["cp", "-r", "/out/.", "/output"]
