use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use commenter_stomp::stomp::StompClientFrame;
use warp::filters::ws::Message;

const EOL_NEW_LINE: &str = "EOL=NewLine";
const EOL_CARRIAGE_RETURN_WITH_NEW_LINE: &str = "EOL=CarriageReturnNewLine";

fn bench_stomp_client_frame_deserialization_disconnect(c: &mut Criterion) {
    let mut group = c.benchmark_group("Disconnect frame deserialization");

    // \n as EOL
    group.bench_with_input(
        BenchmarkId::from_parameter(EOL_NEW_LINE),
        &Message::text("DISCONNECT\n\n\0"),
        |bencher, input| bencher.iter(|| StompClientFrame::new(input)),
    );

    // \r\n as EOL
    group.bench_with_input(
        BenchmarkId::from_parameter(EOL_CARRIAGE_RETURN_WITH_NEW_LINE),
        &Message::text("DISCONNECT\r\n\r\n\0"),
        |bencher, input| bencher.iter(|| StompClientFrame::new(input)),
    );

    group.finish();
}

fn bench_stomp_client_frame_deserialization_subscribe(c: &mut Criterion) {
    let mut group = c.benchmark_group("Subscribe frame deserialization");

    group.bench_with_input(
        BenchmarkId::from_parameter(EOL_NEW_LINE),
        &Message::text("SUBSCRIBE\ndestination:topic-1\nid:sub-1\n\n\0"),
        |bencher, input| bencher.iter(|| StompClientFrame::new(input)),
    );

    group.bench_with_input(
        BenchmarkId::from_parameter(EOL_CARRIAGE_RETURN_WITH_NEW_LINE),
        &Message::text("SUBSCRIBE\r\ndestination:topic-1\r\nid:sub-1\r\n\r\n\0"),
        |bencher, input| bencher.iter(|| StompClientFrame::new(input)),
    );

    group.finish();
}

fn bench_stomp_client_frame_deserialization_unsubscribe(c: &mut Criterion) {
    let mut group = c.benchmark_group("Unsubscribe frame deserialization");

    group.bench_with_input(
        BenchmarkId::from_parameter(EOL_NEW_LINE),
        &Message::text("UNSUBSCRIBE\nid:sub-1\n\n\0"),
        |bencher, input| bencher.iter(|| StompClientFrame::new(input)),
    );
    
    group.bench_with_input(
        BenchmarkId::from_parameter(EOL_CARRIAGE_RETURN_WITH_NEW_LINE),
        &Message::text("UNSUBSCRIBE\r\nid:sub-1\r\n\r\n\0"),
        |bencher, input| bencher.iter(|| StompClientFrame::new(input)),
    );

    group.finish();
}

fn bench_stomp_client_frame_deserialization_send_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("Send create frame deserialization");
    
    group.bench_with_input(
        BenchmarkId::from_parameter(EOL_NEW_LINE),
        &Message::text("SEND\naction:CREATE\ndestination:topic-1\n\npayload...payload..\0"),
        |bencher, input| bencher.iter(|| StompClientFrame::new(input)),
    );
    
    group.bench_with_input(
        BenchmarkId::from_parameter(EOL_CARRIAGE_RETURN_WITH_NEW_LINE),
        &Message::text("SEND\r\naction:CREATE\r\ndestination:topic-1\r\n\r\npayload...payload..\0"),
        |bencher, input| bencher.iter(|| StompClientFrame::new(input)),
    );

    group.finish();
}

fn bench_stomp_client_frame_deserialization_send_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("Send update frame deserialization");
    
    group.bench_with_input(
        BenchmarkId::from_parameter(EOL_NEW_LINE),
        &Message::text("SEND\naction:UPDATE\nid:5ba4c744-1d89-4b32-b2f6-5c7043e12d0b\n\npayload...payload..\0"),
        |bencher, input| bencher.iter(|| StompClientFrame::new(input)),
    );
    
    group.bench_with_input(
        BenchmarkId::from_parameter(EOL_CARRIAGE_RETURN_WITH_NEW_LINE),
        &Message::text("SEND\r\naction:UPDATE\r\nid:5ba4c744-1d89-4b32-b2f6-5c7043e12d0b\r\n\r\npayload...payload..\0"),
        |bencher, input| bencher.iter(|| StompClientFrame::new(input)),
    );

    group.finish();
}

fn bench_stomp_client_frame_deserialization_send_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("Send delete frame deserialization");
    
    group.bench_with_input(
        BenchmarkId::from_parameter(EOL_NEW_LINE),
        &Message::text("SEND\naction:DELETE\nid:5ba4c744-1d89-4b32-b2f6-5c7043e12d0b\n\n\0"),
        |bencher, input| bencher.iter(|| StompClientFrame::new(input)),
    );
    
    group.bench_with_input(
        BenchmarkId::from_parameter(EOL_CARRIAGE_RETURN_WITH_NEW_LINE),
        &Message::text("SEND\r\naction:DELETE\r\nid:5ba4c744-1d89-4b32-b2f6-5c7043e12d0b\r\n\r\n\0"),
        |bencher, input| bencher.iter(|| StompClientFrame::new(input)),
    );

    group.finish();
}

criterion_group!(
    benches,
    bench_stomp_client_frame_deserialization_disconnect,
    bench_stomp_client_frame_deserialization_subscribe,
    bench_stomp_client_frame_deserialization_unsubscribe,
    bench_stomp_client_frame_deserialization_send_create,
    bench_stomp_client_frame_deserialization_send_update,
    bench_stomp_client_frame_deserialization_send_delete
);

criterion_main!(benches);
