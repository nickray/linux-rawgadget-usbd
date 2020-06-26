# linux-rawgadget-usbd

This is an attempt to translate [keyboard.c][keyboard-c] to Rust using the `usb-device` and `nix` libraries.

# Run

This library is being developed on Arch Linux.

- compile and insert `raw_gadget` and `dummy_hcd` kernel modules from [xairy/raw-gadget][upstream]
- run example as root: `sudo -E cargo run --example keyboard`

[keyboard-c]: https://github.com/xairy/raw-gadget/blob/master/examples/keyboard.c
[upstream]: https://github.com/xairy/raw-gadget
