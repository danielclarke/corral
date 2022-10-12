# Corral
## A Simple Sprite Sheet Packer

Corral creates a sprite sheet and data from a folder of images. Corral supports generating json or lua data.

### Usage:
`corral input/to/assets output.png`

### Example Output
![packed sprite sheet](https://github.com/danielclarke/corral/blob/main/assets/demo.png?raw=true)

json excerpt
```json
[{"name":"img_file_name_1","x": 2,"y": 2,"width": 256,"height": 64}...]
```

### Usage, lua data:
`corral test/squares-different-sizes Squares.png --data-fmt=lua`

lua exceprt
```lua
local Squares = {
    RECTANGLE_1 = {
        x = 2,
        y = 2,
        width = 64,
        height = 64,
    },
    RECTANGLE_3 = {
        x = 68,
        y = 2,
        width = 32,
        height = 32,
    },
    RECTANGLE_2 = {
        x = 2,
        y = 68,
        width = 16,
        height = 16,
    },
    RECTANGLE_4 = {
        x = 2,
        y = 86,
        width = 8,
        height = 8,
    }
}

return Squares
```