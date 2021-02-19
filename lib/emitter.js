const { EventEmitter } = require('events');
const { EventEmitter: RustChannel } = require('../native/index.node');


class MyEventEmitter extends EventEmitter {
  constructor() {
    super();

    // Create an instance of the Neon class
    const channel = new RustChannel();

    // Marks the emitter as shutdown to stop iteration of the `poll` loop
    this.isShutdown = false;

    // The `loop` method is called continuously to receive data from the Rust
    // work thread.
    const loop = () => {
      
      // Poll for data
      channel.poll((err, e) => {
        if (err) {
          console.log(err)
          this.emit('error', err);
        } 
        else if (e) {
          const { event, ...data } = e;

          // Emit the event
          this.emit(event, data);
        }
        // Otherwise, timeout on poll, no data to emit

        // Schedule the next iteration of the loop. This is performed with
        // a `setImmediate` to yield to the event loop, to let JS code run
        // and avoid a stack overflow.
        setImmediate(loop);
      });
    };

    // Start the polling loop on next iteration of the JS event loop to prevent zalgo.
    setImmediate(loop);
  }
}

module.exports = MyEventEmitter;
