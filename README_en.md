# waybar-niri-workspaces

A CFFI module for [Waybar](https://github.com/Alexays/Waybar) that displays workspaces and window lists from the [Niri](https://github.com/YaLTeR/niri) compositor in Waybar.
Modified based on [niri_window_buttons](https://github.com/adelmonte/niri_window_buttons), thanks to the original author!

![demo](image.png)

## Features

- **Workspace Display**: Shows all workspaces as labeled tabs
- **Window List**: Displays currently open windows within each workspace with application icons
- **Click Interaction**:
  - Click on a workspace to switch to it
  - Click on a window button to focus the corresponding window
- **Real-time Sync**: Listens to window and workspace changes in real-time via Niri IPC
- **Highly Configurable**: Supports extensive customization options

## Dependencies

- [Rust](https://www.rust-lang.org/) >= 1.85 (for compilation)
- [Waybar](https://github.com/Alexays/Waybar) (with CFFI module support)
- [Niri](https://github.com/YaLTeR/niri) compositor

## Building

```bash
cargo build --release
```

After compilation, the dynamic library will be located at `target/release/libwaybar_niri_workspaces.so`.

## Installation

Copy the compiled `.so` file to your desired location, for example:

```bash
mkdir -p ~/.config/waybar/modules
cp target/release/libwaybar_niri_workspaces.so ~/.config/waybar/modules/
```

## Waybar Configuration

Add the module to your Waybar configuration file (typically `~/.config/waybar/config.jsonc`):

```jsonc
{
  "modules-center": ["cffi/niri_workspaces"],

  "cffi/niri_workspaces": {
    "module_path": "/home/username/.config/waybar/modules/libwaybar_niri_workspaces.so",
    "only_current_workspace": false, // Show only current workspace
    "icon_size": 20,                 // Window icon size (pixels)
    "ignore_rules": [                // Ignore specific windows (optional)
      { "app_id": "firefox" }
    ]
  }
}
```

### Configuration Options

| Option | Type | Default | Description |
|------|------|--------|------|
| `module_path` | string | Required | Absolute path to the `.so` file |
| `only_current_workspace` | boolean | `false` | Show only the current workspace |
| `icon_size` | integer | `24` | Application icon size (pixels) |
| `ignore_rules` | array | `[]` | List of rules to ignore specific windows |

#### `ignore_rules` Rule Fields

| Field | Type | Description |
|------|------|------|
| `app_id` | string | Match window by app_id |
| `title` | string | Exact match window title |
| `title_contains` | string | Window title contains specified string |
| `title_regex` | string | Window title matches regular expression |
| `workspace` | integer | Match specific workspace ID |

## Styling

Add the following styles to your Waybar's `style.css` to customize the appearance:

```css
/************************************************************/
/* cffi/niri_workspaces module styles */

/* Main container */
.niri-workspaces {
}

/* Button base style */
.niri-workspaces button {
    background-color: transparent;
    border: none;
    margin: 0;
    padding: 2px;
    transition: background-color 200ms ease;
}

/* Button hover */
.niri-workspaces button:hover {
    background-color: rgba(255, 255, 255, 0.35);
}

/* Currently focused window button (code uses .focused not .active) */
.niri-workspaces button.focused {
    background-color: rgba(13, 13, 13, 0.5);
}

/* Workspace container */
.niri-workspaces .workspace {
    background-color: rgba(191, 110, 23, 0.7);
    padding: 0;
    margin: 0;
}

/* Active workspace */
.niri-workspaces .workspace.active {
    background-color: rgba(128, 160, 150, 0.95);
}

/* Workspace label */
.niri-workspaces .workspace-label {
    color: rgba(253, 255, 254, 1);
    font-weight: bold;
    font-size: 13px;
    margin-right: 4px;
}
```

### CSS Class Reference

| Class Name | Target | Description |
|------|----------|------|
| `.niri-workspaces` | Main container | Root element of the module |
| `.workspace` | Workspace container | Outer box for each workspace |
| `.workspace.active` | Active workspace | Currently visible workspace |
| `.workspace-label` | Workspace label | Text displaying workspace number |
| `button` | Window button | Button corresponding to each open window |
| `button.focused` | Focused window button | Window with current input focus |
| `button:hover` | Hover state | When mouse is hovering over a button |

## Technical Notes

- This module is written in Rust and interacts with Waybar through `waybar-cffi`
- Uses GTK3 for UI rendering
- Communicates with Niri via `niri-ipc` to listen for window events
- Application icons are resolved through the system icon theme (supports `IconTheme` fallback mechanism)

## License
GPL-3.0 license 