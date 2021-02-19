use neon::prelude::*;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::{self, RecvTimeoutError, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct EventEmitter {
    events: Arc<Mutex<mpsc::Receiver<Event>>>,
    // shutdown: mpsc::Sender<()>,
}
pub struct EventEmitterTask(Arc<Mutex<mpsc::Receiver<Event>>>);

impl Task for EventEmitterTask {
    type Output = Option<Event>;
    type Error = String;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let rx = self
            .0
            .lock()
            .map_err(|_| "Could not obtain lock on receiver".to_string())?;

        // Attempt to read from the channel. Block for at most 100 ms.
        // match rx.recv_timeout(Duration::from_millis(100)) {
        //     Ok(event) => Ok(Some(event)),
        //     Err(RecvTimeoutError::Timeout) => Ok(None),
        //     Err(RecvTimeoutError::Disconnected) => Err("Failed to receive event".to_string()),
        // }
        match rx.recv() {
            Ok(event) => Ok(Some(event)),
            Err(_) => Ok(None),
        }
    }

    fn complete(
        self,
        mut cx: TaskContext,
        event: Result<Self::Output, Self::Error>,
    ) -> JsResult<Self::JsEvent> {
        let event = event.or_else(|err| cx.throw_error(&err.to_string()))?;
        let event = match event {
            Some(event) => event,
            None => return Ok(JsUndefined::new().upcast()),
        };
        let o = cx.empty_object();
        match event.kind {
            // EventKind::Create(notify::event::CreateKind::File) => println!("crate file"),
            // EventKind::Modify(notify::event::ModifyKind::Data(
            //     notify::event::DataChange::Content,
            // )) => {
            //     println!("content notify")
            // }
            EventKind::Modify(notify::event::ModifyKind::Metadata(
                notify::event::MetadataKind::Any,
            )) => {
                let event_name = cx.string("tick");
                let event_count = cx.number(0);

                o.set(&mut cx, "event", event_name)?;
                o.set(&mut cx, "count", event_count)?;
            }
            _ => {
                println!("changed: {:?}", event);
            } // _ => (),
        }
        Ok(o.upcast())
    }
}

fn event_thread() -> mpsc::Receiver<Event> {
    // Create sending and receiving channels for the event data
    let (events_tx, events_rx) = mpsc::channel();

    // Spawn a thead to continue running after this method has returned.
    thread::spawn(move || {
        let (tx, rx) = mpsc::channel();
        let mut watcher: RecommendedWatcher =
            Watcher::new_immediate(move |res| tx.send(res).unwrap()).unwrap();

        watcher
            .watch(
                "/Users/ttang/Practice/log-watcher/log/err.log",
                RecursiveMode::Recursive,
            )
            .unwrap();
        for res in rx {
            match res {
                Ok(event) => {
                    // println!("event: {:?}", event);
                    events_tx.send(event).unwrap();
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });

    events_rx
}

declare_types! {
    pub class JsEventEmitter for EventEmitter {
        init(_) {
            // let (shutdown, shutdown_rx) = mpsc::channel();

            // Start work in a separate thread
            let rx = event_thread();

            // Construct a new `EventEmitter` to be wrapped by the class.
            Ok(EventEmitter {
                events: Arc::new(Mutex::new(rx)),
                // shutdown,
            })
        }

        method poll(mut cx) {
            // The callback to be executed when data is available
            let cb = cx.argument::<JsFunction>(0)?;
            let this = cx.this();

            // Create an asynchronously `EventEmitterTask` to receive data
            let events = cx.borrow(&this, |emitter| Arc::clone(&emitter.events));
            let emitter = EventEmitterTask(events);

            // Schedule the task on the `libuv` thread pool
            emitter.schedule(cb);

            // The `poll` method does not return any data.
            Ok(JsUndefined::new().upcast())
        }
    }
}

register_module!(mut cx, {
    cx.export_class::<JsEventEmitter>("EventEmitter")?;
    Ok(())
});
