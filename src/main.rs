//! This code is base on the Tokio echo server example

#![deny(warnings)]

extern crate tokio;

use tokio::io;
use tokio::net::UnixListener;
use tokio::prelude::*;

use std::env;
use std::path::Path;

fn main() -> Result<(), Box<std::error::Error>> {
    // Allow passing an address to listen on as the first argument of this
    // program.
    let addr = match env::args().nth(1) {
        Some(addr) => addr,
        None => panic!("Please specify path of socket")
    };

    // Next up we create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop, so we pass in a handle
    // to our event loop. After the socket's created we inform that we're ready
    // to go and start accepting connections.
    let socket = UnixListener::bind(Path::new(&addr))?;
    println!("Listening on: {}", addr);

    // Here we convert the `UnixListener` to a stream of incoming connections
    // with the `incoming` method. We then define how to process each element in
    // the stream with the `for_each` method.
    //
    // This combinator, defined on the `Stream` trait, will allow us to define a
    // computation to happen for all items on the stream (in this case TCP
    // connections made to the server).  The return value of the `for_each`
    // method is itself a future representing processing the entire stream of
    // connections, and ends up being our server.
    let done = socket.incoming()
        .map_err(|e| println!("failed to accept socket; error = {:?}", e))
        .for_each(move |socket| {
            // Once we're inside this closure this represents an accepted client
            // from our server. The `socket` is the client connection (similar to
            // how the standard library operates).
            //
            // We just want to copy all data read from the socket back onto the
            // socket itself (e.g. "echo"). We can use the standard `io::copy`
            // combinator in the `tokio-core` crate to do precisely this!
            //
            // The `copy` function takes two arguments, where to read from and where
            // to write to. We only have one argument, though, with `socket`.
            // Luckily there's a method, `Io::split`, which will split an Read/Write
            // stream into its two halves. This operation allows us to work with
            // each stream independently, such as pass them as two arguments to the
            // `copy` function.
            //
            // The `copy` function then returns a future, and this future will be
            // resolved when the copying operation is complete, resolving to the
            // amount of data that was copied.
            let (reader, _) = socket.split();
            let amt = io::copy(reader, tokio::io::stdout());

            // After our copy operation is complete we just print out some helpful
            // information.
            let msg = amt.then(move |result| {
                match result {
                    Ok(_) => (),
                    Err(e) => println!("error: {}", e),
                }

                Ok(())
            });


            // And this is where much of the magic of this server happens. We
            // crucially want all clients to make progress concurrently, rather than
            // blocking one on completion of another. To achieve this we use the
            // `tokio::spawn` function to execute the work in the background.
            //
            // This function will transfer ownership of the future (`msg` in this
            // case) to the Tokio runtime thread pool that. The thread pool will
            // drive the future to completion.
            //
            // Essentially here we're executing a new task to run concurrently,
            // which will allow all of our clients to be processed concurrently.
            tokio::spawn(msg)
        });

    // And finally now that we've define what our server is, we run it!
    //
    // This starts the Tokio runtime, spawns the server task, and blocks the
    // current thread until all tasks complete execution. Since the `done` task
    // never completes (it just keeps accepting sockets), `tokio::run` blocks
    // forever (until ctrl-c is pressed).
    tokio::run(done);
    Ok(())
}
