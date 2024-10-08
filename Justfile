all: build test

build:
    cargo build

build-release:
    cargo build --release --verbose

test:
    cargo test --workspace --all-features --verbose

test-release:
    cargo test --workspace --all-features --release --verbose

coverage:
    cargo tarpaulin --verbose --workspace -o Html

build-time:
    cargo +nightly clean
    cargo +nightly build -Z timings

fmt:
    cargo +nightly fmt -- --check

lint:
    cargo clippy --workspace --all-features --all-targets -- -D warnings

# =================================== #
#      RELEASE STUFF. DO NOT USE      #
# =================================== #

# uses convco to figure out the next version
new_version := "$(convco version --bump)"

release:
    git cliff -t "{{new_version}}" > CHANGELOG.md
    cargo check
    git checkout -b hac-release-v"{{new_version}}"
    git add -A
    git commit -m "chore(release): release hac v{{new_version}}"
    git push --set-upstream origin hac-release-v"{{new_version}}"
    @echo "waiting 10 seconds so github catches up"
    sleep 10
    gh pr create --draft --title "chore: release hac v{{new_version}}" --body "This is an CI auto-generated PR" --reviewer wllfaria
    echo "generated release pull request successfully"

gh-release:
    git tag -d "v{{new_version}}" || echo "tag not found, creating"
    git tag --sign -a "v{{new_version}}" -m "auto generated by the justfile for hac v$(convco version)"
    just cross
    mkdir -p ./target/"release-notes-$(convco version)"
    git cliff -t "v$(convco version)" --current > ./target/"release-notes-$(convco version)/RELEASE.md"
    just checksum >> ./target/"release-notes-$(convco version)/RELEASE.md"
    git push origin "v{{new_version}}"
    gh release create "v$(convco version)" --target "$(git rev-parse HEAD)" --title "hac v$(convco version)" -d -F ./target/"release-notes-$(convco version)/RELEASE.md" ./target/"bin-$(convco version)"/*

checksum:
    @echo "# Checksums"
    @echo "## sha256sum"
    @echo '```'
    @sha256sum ./target/"bin-$(convco version)"/*
    @echo '```'
    @echo "## md5sum"
    @echo '```'
    @md5sum ./target/"bin-$(convco version)"/*
    @echo '```'
    @echo "## blake3sum"
    @echo '```'
    @b3sum ./target/"bin-$(convco version)"/*
    @echo '```'

tar BINARY TARGET:
    tar czvf ./target/"bin-$(convco version)"/{{BINARY}}_{{TARGET}}.tar.gz -C ./target/{{TARGET}}/release/ ./{{BINARY}}

zip BINARY TARGET:
    zip -j ./target/"bin-$(convco version)"/{{BINARY}}_{{TARGET}}.zip ./target/{{TARGET}}/release/{{BINARY}}

tar_static BINARY TARGET:
    tar czvf ./target/"bin-$(convco version)"/{{BINARY}}_{{TARGET}}_static.tar.gz -C ./target/{{TARGET}}/release/ ./{{BINARY}}

zip_static BINARY TARGET:
    zip -j ./target/"bin-$(convco version)"/{{BINARY}}_{{TARGET}}_static.zip ./target/{{TARGET}}/release/{{BINARY}}

binary BINARY TARGET:
    rustup target add {{TARGET}}
    cross build --release --target {{TARGET}}
    just tar {{BINARY}} {{TARGET}}
    just zip {{BINARY}} {{TARGET}}

binary_static BINARY TARGET:
    rustup target add {{TARGET}}
    RUSTFLAGS='-C target-feature=+crt-static' cross build --release --target {{TARGET}}
    just tar_static {{BINARY}} {{TARGET}}
    just zip_static {{BINARY}} {{TARGET}}

binary_no_libgit BINARY TARGET:
    rustup target add {{TARGET}}
    cross build --no-default-features --release --target {{TARGET}}
    just tar {{BINARY}} {{TARGET}}
    just zip {{BINARY}} {{TARGET}}

binary_static_no_libgit BINARY TARGET:
    rustup target add {{TARGET}}
    RUSTFLAGS='-C target-feature=+crt-static' cross build --no-default-features --release --target {{TARGET}}
    just tar_static {{BINARY}} {{TARGET}}
    just zip_static {{BINARY}} {{TARGET}}

cross:
    # Setup Output Directory
    mkdir -p ./target/"bin-$(convco version)"

    rustup toolchain install stable

    ## Linux
    ### x86
    just binary hac x86_64-unknown-linux-gnu
    just binary_static hac x86_64-unknown-linux-gnu
    just binary hac x86_64-unknown-linux-musl
    just binary_static hac x86_64-unknown-linux-musl

    ### aarch
    just binary_no_libgit hac aarch64-unknown-linux-gnu
    just binary_static hac aarch64-unknown-linux-gnu

    ### arm
    just binary_no_libgit hac arm-unknown-linux-gnueabihf
    just binary_static hac arm-unknown-linux-gnueabihf

    ## Windows
    ### x86
    just binary hac.exe x86_64-pc-windows-gnu
    just binary_static hac.exe x86_64-pc-windows-gnu

