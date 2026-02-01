# kino-ui

Kino is a type of resin that is made by eucalyptus trees, and the name of the UI library. 

Built with wgpu and winit, this UI library is inspired by the ui crate [wick3dr0se/egor](https://github.com/wick3dr0se/egor) and uses 
Assembly-like instructions to render different components, including standard and contained widgets.  

# Example

```rust
pub fn init() {
    let renderer = KinoWGPURenderer::new(/*yabba dabba doo*/);
    let windowing = KinoWinitWindowing::new(window.clone());
    
    // keep this with you this is important. the rest are owned by KinoState
    let kino = KinoState::new(renderer, windowing);
}

pub fn update(kino: &mut KinoState) {
    // creating new widget
    let new_rect = kino.add_widget(Box::new(
        Rectangle::new("red patch".into())
            .with(&Rect::new([50.0, 100.0].into(), [128.0, 100.0].into()))
            .colour([255.0, 0.0, 0.0, 255.0]).into()
    ));

    // polling to build the widget tree
    kino.poll();

    // checking input response
    if kino.response(new_rect).clicked {
        println!("I have been clicked!");
    }
}

pub fn render(kino: &mut KinoState) {
    // ...
    
    // generally the last thing to render on your viewport if you want
    // an overlay
    kino.render(/**/);
    
    // ...
}
```