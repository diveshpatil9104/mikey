use mikey::audio::pipeline::JitterBuffer;
use mikey::protocol::{
    read_frame, write_frame, Frame, FrameType, HelloPayload, MediaHeader, WelcomePayload,
    CODEC_PCM, PROTO_VERSION,
};
use mikey::transport::tcp::{configure_stream, handle_client};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

#[test]
fn test_end_to_end_streaming_and_handshake() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local addr");

    let jb = Arc::new(JitterBuffer::new());
    let jb_clone = Arc::clone(&jb);

    let server_thread = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept connection");
        handle_client(stream, jb_clone, "test-pc-id", "test-pc-name");
    });

    // Client connection
    let mut client = TcpStream::connect(addr).expect("connect to server");
    configure_stream(&client).expect("configure client stream");

    // 1. Send HELLO
    let hello = HelloPayload {
        proto: PROTO_VERSION,
        device_id: "test-device-uuid".to_string(),
        device_name: "Pixel 7".to_string(),
        level: 1,
        token: None,
        resume: None,
        caps: vec!["pcm".to_string()],
    };
    let hello_bytes = serde_json::to_vec(&hello).unwrap();
    let hello_frame = Frame::new(FrameType::Hello, hello_bytes);
    write_frame(&mut client, &hello_frame).expect("send hello");

    // 2. Read WELCOME
    let welcome_frame = read_frame(&mut client).expect("read welcome");
    assert_eq!(welcome_frame.frame_type, FrameType::Welcome);
    let welcome: WelcomePayload = serde_json::from_slice(&welcome_frame.payload).unwrap();
    assert_eq!(welcome.pc_id, "test-pc-id");

    // 3. Send AUDIO frame (10 ms PCM = 480 samples = 960 bytes)
    let header = MediaHeader {
        seq: 1,
        capture_ts: 1_700_000_000,
        codec: CODEC_PCM,
        reserved: 0,
    };
    let mut audio_payload = header.to_vec();
    let pcm_samples = vec![1234i16; 480];
    for s in &pcm_samples {
        audio_payload.extend_from_slice(&s.to_le_bytes());
    }
    let audio_frame = Frame::new(FrameType::Audio, audio_payload);
    write_frame(&mut client, &audio_frame).expect("send audio");

    // 4. Send HEARTBEAT
    let ts: u64 = 987_654_321;
    let heartbeat_frame = Frame::new(FrameType::Heartbeat, ts.to_be_bytes().to_vec());
    write_frame(&mut client, &heartbeat_frame).expect("send heartbeat");

    // 5. Read echoed HEARTBEAT
    let echoed_hb = read_frame(&mut client).expect("read echoed heartbeat");
    assert_eq!(echoed_hb.frame_type, FrameType::Heartbeat);
    assert_eq!(echoed_hb.payload, ts.to_be_bytes());

    // 6. Send BYE
    let bye_frame = Frame::new(FrameType::Bye, b"{\"reason\":\"user_stop\"}".to_vec());
    write_frame(&mut client, &bye_frame).expect("send bye");

    server_thread
        .join()
        .expect("server thread terminates cleanly");

    // Jitter buffer reset on disconnect
    assert!(jb.is_empty());
}
