# On The Fly Swap

Swaps an object/trait object from inside an `Arc` on-the-fly, that is, the thread that uses the object won't even notice something happened.

Example:

```rust
use on_the_fly_swap::OnTheFlySwap;

fn main() {
    let mut number_source = OnTheFlySwap::new(Box::new(0));
    let number_source_ = number_source.clone();
    std::thread::spawn(move || loop {
        if let Some(number_source_) = number_source_.lock().inner_mut() {
            *number_source_ += 1;
            println!("value: {}", *number_source_);
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    });
    //Waits a little before substituting the inner object on-the-fly
    std::thread::sleep(std::time::Duration::from_secs(5));
    number_source.replace(Some(Box::new(1000)));
    std::thread::sleep(std::time::Duration::from_secs(5))
}

```

Output:

```
value: 1
value: 2
value: 3
value: 4
value: 5
value: 1001
value: 1002
value: 1003
value: 1004
value: 1005
```

There's support for `parking_lot::Mutex` internally, you just need to activate the `parking_lot` feature. This way there will never be any poisoned locks. Otherwise, `number_source_.lock()` will call `unwrap` internally, because `std::sync::Mutex` is being used.