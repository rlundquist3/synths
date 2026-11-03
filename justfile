build:
    cargo build

vcv-make:
    cargo build
    cd vcv-rack && make install

vcv:
    cargo build
    cd vcv-rack && make install
    osascript -e 'quit app "VCV Rack 2 Free"'
    open /Applications/VCV\ Rack\ 2\ Free.app

docs:
    cargo doc --open --document-private-items