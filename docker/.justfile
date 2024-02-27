build:
    docker build -t test-android .
doctor:
    just build
    docker run --rm -it test-android x doctor
    docker run --rm -it test-android clang --version
test:
    just build
    docker run --name test -it -v $(pwd)/..:/src:ro \
        -v android-test-target:/target \
        -e CARGO_TARGET_DIR=/target \
        -w /src \
        test-android \
        x build --platform android --store play --release --package juggle-mail-android
    docker cp test:/target/x/release/android/juggle-mail-android.aab .
    docker rm test
shell:
    just build
    docker run --rm -it -v $(pwd)/..:/src:ro \
        -v android-test-target:/target \
        -e CARGO_TARGET_DIR=/target \
        -w /src \
        test-android

sign:
    jarsigner -verbose -sigalg SHA256withRSA -digestalg SHA-256 \
        -keystore ~/projects/juggle-mail/juggle.keystore \
        -signedjar signed.aab juggle-mail-android.aab \
        juggle

install:
    rm apks.apks || true
    bundletool build-apks --bundle juggle-mail-android.aab --output apks.apks
    bundletool install-apks --apks apks.apks