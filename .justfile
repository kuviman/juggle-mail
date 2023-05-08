run *OPTIONS:
    cargo run --all-features {{OPTIONS}}

web *OPTIONS:
    cargo geng run --web {{OPTIONS}}

publish:
    cargo geng build --release --web --out-dir dist
    butler push dist kuviman/juggle-mail:html5
