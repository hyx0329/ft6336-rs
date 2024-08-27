# FT6336 driver

This is a simple driver to be used on a M5Stack Core2 rust project.

- Datasheet: <https://focuslcds.com/wp-content/uploads/Drivers/FT6236.pdf>
- Register map from M5Stack: <https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/datasheet/core/Ft6336GU_Firmware%20%E5%A4%96%E9%83%A8%E5%AF%84%E5%AD%98%E5%99%A8_20151112.xlsx>

## Notes

The touch panel used by M5Stack Core2 cannot detect 2 simultaneous touches on the same horizonal line due to a hardware limitation.

## Usage

```rust,ignore
let mut touch_panel = Ft6336::new(SharedI2cBus::new(mutex_i2c_bus));

// the init call ensures the panel in working mode(rather than enginnering mode)
touch_panel.init().unwrap();
// adjust interrupt mode
touch_panel.interrupt_by_pulse().unwrap();

// print current active points
for event in touch_panel.touch_points_iter().unwrap() {
    log::info!(event);
}
```
