# karaconf

## About

Simple Karabiner-Elements manager to sync TOML keymap files under `~/.config/karaconf/<name>.toml`.

## Installation

_(todo...)_

## Usage

```bash
karaconf sync             # compile all ~/.config/karaconf/*.toml into karabiner.json
karaconf switch <name>    # activate a profile (any name from `karaconf list`)
karaconf reset            # alias for `switch system` — the no-rules profile
karaconf list             # list every switchable profile (your TOMLs + `system`)
```

## Example

`~/.config/karaconf/keymap.toml`:

```toml
[settings]
os_layout = "qwerty-us"   # contract for symbol shorthand; only "qwerty-us" is supported today
tap_time  = 200

[macros]
arrow = "->"

[layers.base]
caps_lock     = ["escape", "left_control"]   # tap=esc, hold=ctrl
tab           = ["tab", "nav"]               # tap=tab, hold=nav layer
right_command = "sym"                        # whole-key sym layer trigger

[layers.nav]
h = "left_arrow"
j = "down_arrow"
k = "up_arrow"
l = "right_arrow"

[layers.sym]
n = "$arrow"   # ->
a = "="
```

```bash
karaconf sync          # writes the keymap profile into karabiner.json
karaconf switch keymap
```

### Value types

| Form | Meaning | Example |
|---|---|---|
| `"key"` | Simple remap | `quote = "delete_or_backspace"` |
| `"$macro"` | Macro reference | `d = "$arrow"` |
| `"layer_name"` | Whole-key layer trigger | `right_command = "sym"` |
| `["tap", "hold"]` | Tap-hold | `caps_lock = ["escape", "left_control"]` |
| `["tap", "layer_name"]` | Tap-hold layer trigger | `tab = ["tab", "nav"]` |

Layer names must not shadow Karabiner key codes. Use names like `nav`, `sym`, `func`.

Key expressions support any number of modifier prefixes joined by `+` (`shift+period`, `shift+command+c`) and US-keyboard symbol shorthand (`#`, `|`, …). To use `+` itself as the key in a combo, double it: `command++` = Cmd+Plus. Macros come in three forms: text (`"->"`), explicit sequence (`["hyphen", "shift+period"]`), or repeat (`{ key = "down_arrow", repeat = 5 }`); see `example/keymap.toml`.
