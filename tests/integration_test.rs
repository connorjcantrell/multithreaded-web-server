use hello::thread_pool::ThreadPool;
use hello::server;
use std::net::TcpListener;
use std::thread;
use reqwest;



#[test]
fn simulate() {
    thread::spawn(move || {
        graceful_shutdown();
    });

    let client_result = thread::spawn(move || {
        let client = reqwest::Client::new();
        client.get("127.0.0.1:7878");
        client.get("127.0.0.1:7878");
    });
}

fn graceful_shutdown() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    let mut counter = 0;
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        let handle = pool.execute(|| {
            server::handle_connection(stream);
        });
        counter += handle.join().unwrap();
    }
    println!("Shutting down.");
}