# REQTUI

`reqTUI` is a API Client, much like Postman or Insomnia, but for your terminal.
reqTUI has the goal of providing a good experience for testing APIs without the
need of creating an account, being completely offline, and free and open source.

![Preview](./extra/preview.gif)

<details>
<summary>Expand to see more examples</summary>

> this section will be filled with more examples soon

</details>

## Table of contents
- [Installation](#installation)
- [Documentation](#usage)
- [Customization](#customization)
- [Contributing](#contributing)
- [Changelog](#changelog)

## Installation

This section should guide you through the installation process and how to use
reqTUI.

You can get reqTUI with cargo, or get the latest release
[tag](https://github.com/wllfaria/reqtui/tags).

### Installing with cargo

> [!NOTE]
> you need rust v1.76 or newer

You can get reqTUI from crates.io with:

```sh
cargo install rqt
```

### Building from source

Clone the repository into your machine, and you'll be able to run, or build by 
following the steps below:

```sh
git clone https://github.com/wllfaria/reqtui
cd reqtui
cargo run

# alternatively, you can run:
cargo build --release
# or if you have just:
just build
# the binary will be located at target/release/rqt
```

> [!IMPORTANT]
> reqTUI is in its very early stages of development, new features are added constantly,
> and we have many features planned, feel free to report any bugs, ask for features or
> discuss ideas.

## Documentation

> [!NOTE]
> Documentation is still a work in progress

Documentation can be found in the [reqTUI wiki](https://github.com/wllfaria/reqtui/wiki)

## Customization

Customizing reqTUI is as simple as editing toml files on the config directory, which can
be in different places based on your system and maybe in your environment variables, but
you can run the following command to know where reqTUI is looking for your configuration:

```sh
# this command will print the path to the configuration directory reqTUI is trying to load
rqt --config-dir
```

> [!NOTE]
> You can check all the configuration options and what they mean in the wiki secion
> for customizing reqTUI

reqTUI comes with a set of default configurations, you can check more on the
[wiki](https://github.com/wllfaria/reqtui/wiki), or if you prefer, you can dump the default
configuration and colorscheme to the configuration directory by using:

```sh
rqt --config-dump

# alternatively, you can specify a path

rqt --config-dump <path>
```

## Contributing

All contributions are welcome! Just open a pull request. Please read [CONTRIBUTING.md](./CONTRIBUTING.md)

## Changelog

Changelogs can be found [here](./CHANGELOG.md)
