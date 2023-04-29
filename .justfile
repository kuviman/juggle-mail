run:
    cargo run --all-features

publish:
    cargo geng build --release --web --out-dir dist
    butler push dist kuviman/juggle-mail:html5
