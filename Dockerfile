FROM fedora:25
MAINTAINER Pavel Odvody <podvody@redhat.com>

RUN dnf install -y clang file findutils gcc git llvm redhat-rpm-config tar \
    {clang,zlib}-devel openssl{,-devel}
RUN useradd fedora
COPY . /opt/pixton
RUN chown -R fedora:fedora /opt/pixton

USER fedora
WORKDIR /home/fedora
ENV PATH="/home/fedora/.cargo/bin:$PATH" USER='fedora'
RUN curl https://sh.rustup.rs -sSf > rustup\
 && bash rustup -y --default-toolchain stable\
 && (cd /opt/pixton && cargo build --release)

ENV RUST_BACKTRACE=1
WORKDIR /opt/pixton
CMD ["target/release/pixton"]
LABEL io.hica.bind_pwd=1\
      io.hica.command_aliases='{"build": {"cmd": "rustup run stable cargo build", "synopsis": "none"}}'\
      io.hica.tty=1
EXPOSE 6868
