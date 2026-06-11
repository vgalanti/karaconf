# karaconf

Simple Karabiner-Elements manager to sync TOML keymap files under `~/.config/karaconf/<name>.toml`.

## Install

```bash
brew install vgalanti/tap/karaconf
```

from source (requires Karabiner-Elements to be installed)

```bash
cargo install --git https://github.com/vgalanti/karaconf
```

## Usage

```bash
karaconf sync             # compile all ~/.config/karaconf/*.toml into karabiner.json
karaconf list             # list switchable profiles (TOML profiles + `system`)
karaconf switch <name>    # activate a profile from `karaconf list`
karaconf reset            # alias for `switch system` — OS default layout
```

## Config

profiles live at `~/.config/karaconf/<name>.toml`.

```toml
[settings]
os_layout  = "qwerty-us"   # symbol-shorthand contract; only "qwerty-us" supported
tap_time   = 100           # tap-hold window, ms
combo_time = 50            # combo simultaneous-press window, ms

[macros]
arrow     = "->"
five_down = { key = "down_arrow", repeat = 5 }
bol       = "{left_command+left_arrow}"

[layers.base]
caps_lock     = ["escape", "left_control"]   # tap=esc, hold=ctrl
tab           = ["tab", "nav"]               # tap=tab, hold=nav layer
right_command = "sym"                        # whole-key sym layer trigger

[layers.nav]
h = "left_arrow"
j = "down_arrow"

[layers.sym]
n = "$arrow"
a = "="
```

```bash
karaconf sync            # write profile into karabiner.json
karaconf switch keymap
```

`base` is always active. layer names must not shadow Karabiner key codes — use `nav`, `sym`, `func`.

### Value types

| Form | Meaning | Example |
|---|---|---|
| `"key"` | remap | `quote = "delete_or_backspace"` |
| `"$macro"` | macro reference | `d = "$arrow"` |
| `"layer_name"` | whole-key layer trigger | `right_command = "sym"` |
| `["tap", "hold"]` | tap-hold | `caps_lock = ["escape", "left_control"]` |
| `["tap", "layer_name"]` | tap-hold layer trigger | `tab = ["tab", "nav"]` |

key expressions:

- modifiers: join with `+` (`shift+command+c`)
- symbols: US-keyboard shorthand (`#`, `|`)

### Combos

a base-layer key with `+` declares a chord: fires when all parts pressed within `combo_time`.

```toml
[layers.base]
"j+k" = "escape"   # chord -> key
"d+f" = "nav"      # chord triggers a layer (base only)
"u+i" = "$arrow"   # chord runs a macro
```

### Macros

| Form | Example |
|---|---|
| text | `arrow = "->"` |
| non-character keys | `bol = "{left_command+left_arrow}"` |
| sequence | `delete = ["hyphen", "shift+period"]` |
| repeat | `five_down = { key = "down_arrow", repeat = 5 }` |

reference with `$name`. see `example/keymap.toml`.
