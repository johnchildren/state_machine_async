# state-machine-async


Experimenting with a [state_machine_future](https://github.com/fitzgen/state_machine_future) alternative using async/await.

I'm trying to work out if it's still worth writing state machines through derive macros,
so I wrote up one of the example state machines from `state_machine_future` in order to see what the ergonomics would be like.
It seemed okay and potentially worthwhile so the next stage is writing the derive crate.


Requires Rust Nightly 1.39+

