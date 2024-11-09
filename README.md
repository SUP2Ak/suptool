# suptool

![GitHub Rust](https://github.com/sup2ak/suptool/actions/workflows/rust.yml/badge.svg)
![GitHub Deno](https://github.com/sup2ak/suptool/actions/workflows/deno.yml/badge.svg)

__Documentation:__ [:gb:](./README.md) | [:fr:](./README.fr.md)

## Description

Suptool is a desktop application developed with Tauri, React, and TypeScript. It
allows you to search and index files on your system, providing a modern and
responsive user interface.

## Prerequisites

Before you begin, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install) (stable version)
- [Node.js](https://nodejs.org/) (version 18 or higher)
- [pnpm](https://pnpm.io/installation) for package management
- [Deno v2](https://deno.land/manual/getting_started/installation) to run
  scripts
- [Tauri CLI v2](https://v2.tauri.app/) to
  build and run the application

## Installation

1. **Clone the repository:**

   ```bash
   git clone https://github.com/your-username/suptool.git
   cd suptool
   ```

2. **Install Deno dependencies:**

   Run the following command to install the dependencies:

   ```bash
   deno task setup
   ```

3. **Start the application in development mode:**

   Run the following command in the root directory:

   ```bash
   deno task tauri dev
   ```

   This will launch the application and open a browser window for development.

4. **Build the application:**

   To build the application, use the following command:

   ```bash
   deno task tauri build
   ```

   Or, if you want to specify the runner and target:

   ```bash
   deno task tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc
   ```

## Usage

Once the application is launched, you can:

- Index your files by clicking the indexing button.
- Search for files using the search bar.
- And much more to come!

## Contributing

Contributions are welcome! If you would like to contribute, please follow these
steps:

## License

This project is licensed under the MIT License. For more details, please refer
to the [LICENSE](LICENSE) file.

## Help

For any questions or issues, feel free to open an issue on the GitHub
repository.
