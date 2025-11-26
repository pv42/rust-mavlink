#![feature(test)]

extern crate test;

#[cfg(all(feature = "default", feature = "ardupilotmega"))]
mod process_files {
    use mavlink::ardupilotmega::MavMessage;
    use mavlink::error::MessageReadError;

    #[test]
    pub fn get_file() {
        // Get path for download script
        let tlog = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/log.tlog")
            .canonicalize()
            .unwrap();

        let tlog = tlog.to_str().unwrap();

        let filename = std::path::Path::new(tlog);
        let filename = filename.to_str().unwrap();
        dbg!(filename);

        println!("Processing file: {filename}");
        let connection_string = format!("file:{filename}");

        // Process file
        process_file(&connection_string);
    }

    pub fn process_file(connection_string: &str) {
        let vehicle = mavlink::connect::<MavMessage>(connection_string);
        assert!(vehicle.is_ok(), "Incomplete address should error");

        let vehicle = vehicle.unwrap();
        let mut counter = 0;
        loop {
            match vehicle.recv() {
                Ok((_header, _msg)) => {
                    counter += 1;
                }
                Err(MessageReadError::Io(e)) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        continue;
                    } else {
                        println!("recv error: {e:?}");
                        break;
                    }
                }
                _ => {}
            }
        }

        println!("Number of parsed messages: {counter}");
        assert!(
            counter == 1426,
            "Unable to hit the necessary amount of matches"
        );
    }

    use std::io::Read;
    use test::Bencher;

    // test process_files::bench_read_any_msg ... bench: 230,560,580.00 ns/iter (+/- 1,034,277.00)
    // test process_files::bench_read_any_msg ... bench: 229,439,510.00 ns/iter (+/- 943,793.00)

    //
    // 231,425.00 ns/iter (+/- 2,482.50)
    // 232,786.67 ns/iter (+/- 2,487.33)
    #[bench]
    fn bench_read_any_msg(bench: &mut Bencher) {
        // Get path for download script
        let tlog = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/log.tlog")
            .canonicalize()
            .unwrap();

        let tlog = tlog.to_str().unwrap();

        let filename = std::path::Path::new(tlog);

        let mut file = std::fs::File::open(filename).unwrap();
        let mut buf = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buf).unwrap();

        bench.iter(|| process_data(&buf));
    }

    fn process_data(data: &[u8]) {
        use mavlink::peek_reader::PeekReader;
        let mut reader = PeekReader::new(data);
        let mut counter = 0;
        while let Ok((head, body)) = mavlink::read_any_msg::<MavMessage, _>(&mut reader) {
            std::hint::black_box(head);
            std::hint::black_box(body);
            counter += 1;
        }
        assert_eq!(counter, 1426);
    }
}
