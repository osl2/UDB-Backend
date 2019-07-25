# upowdb backend

upowdb is an educational tool for learning to work with relational databases.
This is the backend API that stores teacher accounts and course data.

## Installation

Use the package manager [cargo](https://doc.rust-lang.org/cargo/index.html) and git to install the upowdb backend.
You also need to have development version of the sqlite, libpq and libgcc libraries installed.

```bash
git clone https://git.scc.kit.edu/pse-lernplattform-datenbanken/backend.git upowdb-backend
cd upowdb-backend
cargo install
```

## Usage

```bash
> upowdb-backend --help

upowdb-backend 0.1.0
Jan Christian Grünhage <jan.christian@gruenhage.xyz>:David Lamm <david-lamm@web.de>


USAGE:
    udb-back [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -v, --verbose    Be verbose (you can add this up to 4 times for more logs)
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config>    Set config file path
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License
[AGPLv3](https://choosealicense.com/licenses/agpl-3.0/)
