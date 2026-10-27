# RustCraft Modding Guide & Tutorial

Welcome to the **RustCraft Modding Guide**! RustCraft features a zero-dependency, data-driven modding system. You can create custom blocks, items, food, tools, crafting recipes, furnace smelting, mob drop tables, and in-game chat commands—**without needing a Rust compiler or external tools**.

All mods are placed in the `mods/` directory and use simple JSON definitions.

---

## 1. Quick Start

1. Open the RustCraft directory.
2. Locate (or create) the `mods/` folder next to your executable:
   ```
   rustcraft/
   ├── mods/
   │   └── my_first_mod/
   │       └── mod.json
   ├── saves/
   └── Cargo.toml
   ```
3. Create a folder for your mod (e.g. `mods/my_first_mod/`).
4. Create a `mod.json` file inside that folder.
5. Start or restart RustCraft. Your mod will be automatically discovered and loaded!

---

## 2. Directory Layout & Manifest

Every mod requires a `mod.json` file in its folder. The root structure looks like this:

```json
{
  "id": "my_first_mod",
  "name": "My First Mod",
  "version": "1.0.0",
  "author": "YourName",
  "description": "Adds awesome new blocks, items, recipes, and commands!",

  "blocks": [ /* Custom blocks */ ],
  "items": [ /* Custom items / food */ ],
  "recipes": [ /* Crafting table recipes */ ],
  "smelting": [ /* Furnace recipes */ ],
  "loot": [ /* Mob drops */ ],
  "commands": [ /* Custom chat commands */ ]
}
```

> **Tip:** You can write `// comments` inside your `mod.json`! RustCraft's built-in parser automatically ignores single-line comments.

---

## 3. Creating Custom Blocks

Blocks are 3D voxels placed in the world, collision-checked, meshed, and lit.

### Block Attributes
- `id` *(number, required)*: Numerical ID from **140 to 255** (IDs 0–139 are reserved for vanilla blocks).
- `identifier` *(string, required)*: Unique string identifier (e.g. `"ruby_ore"`). Used in recipes, commands, and save files.
- `name` *(string)*: Display name shown in tooltips, inventory, and chat.
- `solid` *(boolean, default: `true`)*: Enables physical bounding box collision.
- `opaque` *(boolean, default: `true`)*: Whether the block occludes light and culls adjacent faces.
- `transparent` *(boolean, default: `false`)*: For see-through materials like glass or ice.
- `hardness` *(number, default: `1.5`)*: Breaking time in seconds with bare hands. Bedrock uses `-1.0` (unbreakable).
- `light` *(number, 0 to 15, default: `0`)*: Ambient light emitted by the block (e.g. `14` for torch/lava, `4` for glowing mineral).
- `drops` *(object, optional)*: What item drops when the block is mined. Defaults to the block itself.
  - `item` *(string)*: Identifier of the dropped item/block.
  - `count` *(number, default: `1`)*: Number of items dropped.

### Example Block Definition
```json
{
  "id": 140,
  "identifier": "emerald_ore",
  "name": "Emerald Ore",
  "solid": true,
  "opaque": true,
  "hardness": 3.0,
  "light": 0,
  "texture": {
    "pattern": "ore",
    "background_color": [0.55, 0.55, 0.55],
    "ore_color": [0.05, 0.95, 0.35]
  },
  "drops": {
    "item": "emerald",
    "count": 1
  }
}
```

---

## 4. Creating Custom Items & Food

Items exist in the player's inventory, hotbar, and dropped in the world as 3D floating items, but are not placeable as solid terrain blocks.

### Item Attributes
- `id` *(number, required)*: Numerical ID from **140 to 255** (IDs 0–139 are reserved for vanilla blocks and items).
- `identifier` *(string, required)*: Unique item identifier (e.g. `"emerald"`, `"ruby_apple"`).
- `name` *(string)*: Display name in inventory and chat.
- `is_food` *(boolean, default: `false`)*: When `true`, holding right-click eats the item.
- `nutrition` *(array `[hunger, saturation]`, optional)*:
  - `hunger`: Hunger points restored (e.g. `4.0` restores 2 drumsticks; max player hunger is `20.0`).
  - `saturation`: Saturation bonus points (keeps hunger full longer).

### Example Item & Food Definition
```json
{
  "id": 142,
  "identifier": "emerald",
  "name": "Emerald",
  "is_food": false,
  "texture": {
    "pattern": "gem",
    "color": [0.05, 0.95, 0.35]
  }
},
{
  "id": 143,
  "identifier": "emerald_apple",
  "name": "Emerald Apple",
  "is_food": true,
  "nutrition": [10.0, 16.0],
  "texture": {
    "pattern": "apple",
    "color": [0.1, 0.95, 0.4]
  }
}
```

---

## 5. Procedural Textures & Patterns

RustCraft features high-performance procedural pixel-art synthesis. You don't need image files! Simply choose a `pattern` and set colors.

Colors can be specified as RGB floats `[r, g, b]` (0.0 to 1.0) or hex strings (e.g. `"#00ff7f"`).

| Pattern | Description | Config Parameters |
|---|---|---|
| `"ore"` | Classic stone background with flecks of mineral | `background_color`, `ore_color` |
| `"mineral"` | Solid mineral block with shaded beveled border | `base_color`, `border_color` |
| `"speckled"` | Textured surface with deterministic noise variance | `base_color`, `variance` (e.g. `0.15`) |
| `"gem"` | Diamond/cut gem facet icon | `color` |
| `"apple"` | Fruit shape with stem and leaf | `color` |
| `"ingot"` | Metallic bar ingot | `color` |
| `"sword"` | 16x16 pixel-art blade and hilt | `blade_color`, `handle_color` |
| `"pickaxe"` | 16x16 curved pickaxe head on shaft | `head_color`, `handle_color` |

---

## 6. Crafting Recipes

Recipes support **1x1**, **2x2** (inventory crafting), and **3x3** (crafting table) shaped grids. Empty spaces are represented by `"_"` or `"air"`.

### Example Recipes
```json
"recipes": [
  // 3x3 Block Crafting: 9 emeralds -> 1 Emerald Block
  {
    "type": "crafting",
    "grid": [
      ["emerald", "emerald", "emerald"],
      ["emerald", "emerald", "emerald"],
      ["emerald", "emerald", "emerald"]
    ],
    "result": "emerald_block",
    "count": 1
  },

  // 1x1 Reverse Crafting: 1 Emerald Block -> 9 emeralds
  {
    "type": "crafting",
    "grid": [
      ["emerald_block"]
    ],
    "result": "emerald",
    "count": 9
  },

  // Surrounded Recipe: 8 emeralds around 1 vanilla apple -> 1 Emerald Apple
  {
    "type": "crafting",
    "grid": [
      ["emerald", "emerald", "emerald"],
      ["emerald", "apple",   "emerald"],
      ["emerald", "emerald", "emerald"]
    ],
    "result": "emerald_apple",
    "count": 1
  }
]
```

---

## 7. Furnace Smelting Recipes

Furnace recipes turn an input item into an output item when cooked over fuel (coal, wood, etc.).

```json
"smelting": [
  {
    "input": "emerald_ore",
    "output": "emerald"
  },
  {
    "input": "ruby_ore",
    "output": "ruby"
  }
]
```

---

## 8. Mob Defeat Loot Tables

You can add custom loot drops when hostile or passive mobs are killed.

Supported targets: `"zombie"`, `"skeleton"`, `"creeper"`, `"pig"`, `"cow"`, `"chicken"`, `"sheep"`, `"dolphin"`, `"cod"`, `"salmon"`, `"pufferfish"`, `"tropical_fish"`.

```json
"loot": [
  {
    "target": "zombie",
    "item": "emerald",
    "count": 1,
    "chance": 0.35 // 35% probability per zombie defeat
  },
  {
    "target": "skeleton",
    "item": "ruby",
    "count": 1,
    "chance": 0.40 // 40% probability per skeleton defeat
  }
]
```

---

## 9. Custom In-Game Chat Commands

Mods can register custom slash commands (e.g. `/emerald`, `/ruby`, `/magic_heal`). When a player types the command in chat, its actions are executed sequentially.

### Available Action Types
- `message` / `echo`: Displays a message in chat (`"text": "..."`).
- `give`: Adds items to player inventory (`"item": "...", "count": 16`).
- `heal`: Restores player to full health, hunger, and saturation.
- `fly`: Toggles creative flight mode on or off.
- `time`: Sets the day/night cycle time (`"ticks": 1000` for day, `13000` for night).
- `spawn`: Spawns a mob at the player's position (`"mob": "pig"`, `"count": 3`).
- `clear`: Clears the player's inventory.

### Example Commands
```json
"commands": [
  {
    "name": "emerald",
    "description": "Grants 16 emeralds and an Emerald Apple",
    "actions": [
      { "type": "give", "item": "emerald", "count": 16 },
      { "type": "give", "item": "emerald_apple", "count": 1 },
      { "type": "message", "text": "Received Emerald Starter Pack!" }
    ]
  },
  {
    "name": "magic_heal",
    "description": "Restores full health and hunger",
    "actions": [
      { "type": "heal" },
      { "type": "message", "text": "Emerald magic has restored your vitality!" }
    ]
  }
]
```

---

## 10. In-Game Testing & Verification

1. **Check Logs at Startup:**
   When starting RustCraft, look at your console or log file:
   ```
   [MODS] Scanning 'mods' for mods...
   [MODS] Successfully loaded mod 'Emerald & Ruby Expansion' [emerald_expansion] v1.0.0 by Shieldziak & RustCraft Team
   [MODS] Mod loading complete. 1 mod(s) active.
   ```
2. **Creative Mode Inventory:**
   Press `E` in Creative Mode:
   - Your custom blocks will appear in the **BLOCKS** tab.
   - Your custom items/tools will appear in the **TOOLS** tab.
   - All custom content appears in the **ALL** tab.
3. **Chat Commands:**
   Press `T` or `/` in-game:
   - Type `/help` to see your mod's registered commands listed under `--- Mod Commands ---`.
   - Test giving yourself items: `/give emerald 64` or `/give emerald_ore 10`.
   - Test your custom command: `/emerald` or `/magic_heal`.
4. **Smelting & Crafting:**
   - Place a Crafting Table and verify your 3x3 recipes.
   - Place a Furnace, insert coal and your custom ore, and verify output smelting.

---

## 11. Multiplayer Automatic Mod Downloading (Source Engine Style)

RustCraft features a built-in network resource synchronization system inspired by **Valve Source engine** games (Counter-Strike, Garry's Mod, Team Fortress 2):

### How It Works
1. **Server Setup:**
   - Any mods placed in the dedicated server's `./mods/` directory are loaded when the server starts.
   - When a player connects to the server, the server automatically transmits all active mod definitions via the `ServerPacket::ModSync` packet during the handshake sequence before spawning into the world.
2. **Client Auto-Download:**
   - Connecting players automatically receive and download the server's mods over the TCP connection.
   - The files are saved locally to `./mods/<mod_name>/mod.json`, meaning players keep them permanently for future sessions and offline play.
   - If the server has updated a mod with new blocks or recipes, the client detects the change and replaces its local copy.
3. **Seamless Engine Reload:**
   - As soon as the mods are downloaded, the client runtime automatically registers the new blocks, items, recipes, and drop tables.
   - The texture atlas and GPU texture arrays are automatically updated without restarting the game.
   - When the player enters the world, all custom server blocks placed by other players are already textured and interactable!

> [!TIP]
> Players do **not** need to manually download or install any files to join a modded server. Everything is 100% automated!

---

## 12. Complete Working Example

RustCraft comes with a pre-installed reference mod located at:
`mods/emerald_expansion/mod.json`

You can inspect or copy this mod to jumpstart your own creations!

