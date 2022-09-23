# Corral
## A Basic Sprite Sheet Packer

Corral creates a sprite sheet and json data from a folder of images.

### Usage:
`corral input/to/assets output.png`

### Example Output
![packed sprite sheet](https://github.com/danielclarke/corral/blob/main/assets/demo.png?raw=true)

Json excerpt

```json
[{
    name: img_file_name_1
    x: 2,
    y: 2,
    width: 256,
    height: 64
}
...
]
```