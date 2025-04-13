# GOA CLI

The official command-line interface for [Go on Airplanes](https://github.com/kleeedolinux/goonairplanes) - a Go-based fullstack framework with HTML file-based routing.

## Features

- Create new Go on Airplanes projects
- Generate API routes with intelligent defaults
- Manage page routes with dynamic parameters
- Create reusable components
- Build and configure your applications
- List and manage project resources
- Developer-focused experience with interactive prompts

## Installation

### From Binary Releases

Download the latest binary for your platform from the [releases page](https://github.com/goonairplanes/goa-cli/releases).

#### Linux/macOS
```bash
chmod +x goa
sudo mv goa /usr/local/bin/
```

#### Windows
Add the directory containing the goa.exe file to your PATH.

### From Source

```bash
git clone https://github.com/goonairplanes/goa-cli.git
cd goa-cli
cargo build --release
```

The binary will be available in `target/release/goa` (or `target\release\goa.exe` on Windows).

## Usage

### Project Commands

```bash
# Create a new project
goa project new

# List all routes and components in your project
goa project list

# Configure your project settings
goa project config

# Build your project (with optional output path)
goa project build
goa project build --output ./dist
```

### Route Commands

```bash
# Create a new API route
goa route api new users/auth/login

# Delete an API route
goa route api delete users/auth/login

# Create a new page route
goa route page new dashboard

# Delete a page route
goa route page delete dashboard
```

### Component Commands

```bash
# Create a new component
goa component new card

# Delete a component
goa component delete card
```

## Project Structure

When you create a new project with `goa project new`, it will set up a standard Go on Airplanes project structure with:

- `/app` - Contains your application code
  - `/api` - API routes and handlers
  - `/pages` - HTML page routes
  - `/components` - Reusable HTML components
- `config.json` - Project configuration
- `main.go` - Application entry point

## License

MIT 