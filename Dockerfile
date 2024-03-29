FROM fedora:25
MAINTAINER Pavel Odvody <podvody@redhat.com>

RUN dnf install -y clang file findutils gcc git llvm redhat-rpm-config tar \
    {clang,zlib}-devel openssl{,-devel}
RUN useradd fedora --gid 0
COPY . /opt/pixton
RUN chown -R fedora:0 /opt/pixton

USER fedora
WORKDIR /home/fedora
ENV PATH="/home/fedora/.cargo/bin:$PATH" USER='fedora'
RUN curl https://sh.rustup.rs -sSf > rustup\
 && bash rustup -y --default-toolchain stable\
 && (cd /opt/pixton && cargo build --release)

RUN chgrp -R 0 /opt/pixton && chmod -R g+rw /opt/pixton\
 && find /opt/pixton -type d -exec chmod g+x {} +

ENV RUST_BACKTRACE=1
WORKDIR /opt/pixton
CMD ["target/release/pixton"]
LABEL io.hica.bind_pwd=1\
      io.hica.command_aliases='{"build": {"cmd": "rustup run stable cargo build", "synopsis": "none"}}'\
      io.hica.tty=1
EXPOSE 6868
