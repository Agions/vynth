use criterion::{black_box, criterion_group, criterion_main, Criterion};
use synerix::session::{Session, SessionStore, StoredMessage};

fn bench_session_create_and_list(c: &mut Criterion) {
    c.bench_function("session/create_and_list_100", |b| {
        b.iter(|| {
            let store = SessionStore::memory().expect("create memory store");

            for i in 0..100 {
                let session =
                    Session::new(black_box(&format!("Session {}", i)), black_box("gpt-4"));
                store.create_session(black_box(&session)).unwrap();
            }

            let sessions = store.list_sessions().unwrap();
            black_box(sessions.len());
        })
    });
}

fn bench_message_save_and_load(c: &mut Criterion) {
    c.bench_function("session/save_and_load_50_messages", |b| {
        b.iter(|| {
            let store = SessionStore::memory().expect("create memory store");
            let session = Session::new(black_box("Bench Session"), black_box("gpt-4"));
            store.create_session(black_box(&session)).unwrap();

            for i in 0..50 {
                let msg = StoredMessage::user(
                    black_box(&session.id),
                    black_box(&format!(
                        "Message number {} with some content for benchmarking purposes",
                        i
                    )),
                );
                store.save_message(black_box(&msg)).unwrap();
            }

            let messages = store.load_messages(black_box(&session.id)).unwrap();
            black_box(messages.len());
        })
    });
}

criterion_group!(
    benches,
    bench_session_create_and_list,
    bench_message_save_and_load
);
criterion_main!(benches);
