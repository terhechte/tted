# Tted

## A rich text library for [Forma](https://github.com/google/forma)

This is a proof of concept of rendering rich text on the GPU using Forma, including Emoji (which are rendered as textures).

## Current status

- The CPU renderer performs far better than the GPU renderer
- Clipping improves performance but currently there seem to be artifacts, not sure what I'm doing wrong
- Memory usage is huge when using the GPU renderer
- Large amounts of text are rendering quite well on the CPU though.
- I'm probably wrong about all kinds of assumptions I made when building this :-)

## Media



https://user-images.githubusercontent.com/132234/208634357-70b275d6-736a-4714-82de-661e20205b03.mov



## Try it out

``` toml
tted = { git = "https://github.com/terhechte/tted" }
```

Or run the example:

``` sh
cargo run --release --example demo
```
