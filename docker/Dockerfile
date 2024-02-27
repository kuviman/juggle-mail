FROM rust

RUN rustup target add aarch64-linux-android armv7-linux-androideabi
RUN cargo install --git https://github.com/kuviman/xbuild

RUN apt-get update; apt-get install -y pkg-config libx11-dev libasound2-dev libudev-dev lld llvm
RUN apt-get install -y clang

RUN apt-get install wget
RUN wget https://services.gradle.org/distributions/gradle-7.6.4-bin.zip && \
    unzip -d /opt/gradle gradle-7.6.4-bin.zip
ENV PATH=/opt/gradle/gradle-7.6.4/bin:$PATH

RUN apt-get install -y openjdk-17-jre
RUN apt-get install -y android-sdk
ENV ANDROID_HOME=/usr/lib/android-sdk
RUN apt-get install -y sdkmanager
RUN yes | sdkmanager --licenses

RUN cargo install --git https://github.com/kuviman/xbuild && true && true