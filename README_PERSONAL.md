# static lifetime 
```rust
fn init_display(
        i2c0: I2C0,
        sda: Gpio21,
        scl: Gpio22,
    ) -> Result<GraphicsMode<I2cInterface<I2cDriver<'static>>>> {
        let config = I2cConfig::new().baudrate(Hertz(100_000)).into();
        let i2c = I2cDriver::new(i2c0, sda, scl, &config)?;
        let mut display: GraphicsMode<I2cInterface<I2cDriver>> =
            Builder::new().connect_i2c(i2c).into();
        display.init().expect("fail to init display");

        Ok(display)
    }

```

If we don't specify 'static as the lifetime for the I2cDriver in the init_display function, it would mean that the I2cDriver and any references it contains would have a lifetime tied to the scope of the function where it's created. This would imply that the I2cDriver instance and its associated references can only be used within the function init_display, and attempting to return them would result in a compiler error.

However, in this case, we intend to return the GraphicsMode<I2cInterface<I2cDriver>> from the function, which means it needs to be valid outside the function's scope. Since we're creating the I2cDriver instance within the function and then returning the GraphicsMode that contains it, specifying 'static as the lifetime ensures that the returned GraphicsMode can be used outside the function and will be valid for the entire duration of the program.

In short: since we return a result type, it will contain a reference to the the graphicMode. However, if no lifetime is specified, the GraphicMode is "dropped" and the Ok() variant of our result will contain a dangling reference. Specifying 'static as the lifetime ensures that the returned value can live for the entire program duration, and this is exactly how the borrow checker makes sure references are valid!


