use puce16_desktop::audio::*;

#[test]
fn e08_15_new_is_empty() {
    let ring = AudioRing::new();
    assert_eq!(ring.len(), 0);
    assert_eq!(ring.capacity(), 8192);
    assert_eq!(ring.free(), 8192);
}

#[test]
fn e08_15_push_pop_order() {
    let mut ring = AudioRing::new();
    let frames: [[i16; 2]; 3] = [[1, 1], [2, 2], [3, 3]];
    assert_eq!(ring.push(&frames), 3);

    let mut out = [[0, 0]; 2];
    assert_eq!(ring.pop(&mut out), 2);
    assert_eq!(out[0], [1, 1]);
    assert_eq!(out[1], [2, 2]);
    assert_eq!(ring.len(), 1);

    let mut out2 = [[0, 0]; 5];
    assert_eq!(ring.pop(&mut out2), 1);
    assert_eq!(out2[0], [3, 3]);
}

#[test]
fn e08_15_push_full() {
    let mut ring = AudioRing::new();
    let frames: Vec<[i16; 2]> = (0..8192).map(|i| [i as i16, -(i as i16)]).collect();
    assert_eq!(ring.push(&frames), 8192);

    let extra: [[i16; 2]; 10] = [[99, 99]; 10];
    assert_eq!(ring.push(&extra), 0);
    assert_eq!(ring.len(), 8192);

    let mut out = [[0, 0]; 4];
    ring.pop(&mut out);
    assert_eq!(ring.push(&extra), 4);
}

#[test]
fn e08_15_wraparound() {
    let mut ring = AudioRing::new();
    let frames: Vec<[i16; 2]> = (0..6000).map(|i| [i as i16, -(i as i16)]).collect();
    assert_eq!(ring.push(&frames), 6000);

    let mut out = [[0, 0]; 6000];
    assert_eq!(ring.pop(&mut out), 6000);

    // Push again to force wraparound past end of buffer
    let frames2: Vec<[i16; 2]> = (0..6000).map(|i| [i as i16, -(i as i16)]).collect();
    assert_eq!(ring.push(&frames2), 6000);

    let mut out2 = [[0, 0]; 6000];
    assert_eq!(ring.pop(&mut out2), 6000);
    for (i, frame) in out2.iter().enumerate() {
        assert_eq!(*frame, [i as i16, -(i as i16)]);
    }
}

#[test]
fn e08_15_pop_empty() {
    let mut ring = AudioRing::new();
    let mut out = [[99, 99]; 3];
    assert_eq!(ring.pop(&mut out), 0);
    // Output buffer should be unchanged
    for frame in &out {
        assert_eq!(*frame, [99, 99]);
    }
}

#[test]
fn e08_15_clear() {
    let mut ring = AudioRing::new();
    let frames: [[i16; 2]; 3] = [[1, 1], [2, 2], [3, 3]];
    assert_eq!(ring.push(&frames), 3);
    ring.clear();
    assert_eq!(ring.len(), 0);

    // Push/pop should still work after clear
    let frames2: [[i16; 2]; 2] = [[4, 4], [5, 5]];
    assert_eq!(ring.push(&frames2), 2);
    let mut out = [[0, 0]; 2];
    assert_eq!(ring.pop(&mut out), 2);
    assert_eq!(out[0], [4, 4]);
}

#[test]
fn e08_15_shared_ring_threads() {
    let ring = new_shared_ring();
    let total_frames = 5000;
    const BLOCK_SIZE: usize = 500;
    let num_blocks = 10;

    // Producer thread: write 10 blocks of 500 frames each
    let producer = {
        let ring = ring.clone();
        std::thread::spawn(move || {
            for block in 0..num_blocks {
                let start = block * BLOCK_SIZE;
                let frames: Vec<[i16; 2]> = (start..start + BLOCK_SIZE)
                    .map(|i| [i as i16, -(i as i16)])
                    .collect();

                // Retry until all frames in this block are pushed
                let mut written = 0;
                while written < BLOCK_SIZE {
                    let r = ring.lock().unwrap().push(&frames[written..]);
                    if r == 0 {
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    } else {
                        written += r;
                    }
                }
            }
        })
    };

    // Consumer: read until we have all frames
    let mut received = 0;
    while received < total_frames {
        let mut out = [[0, 0]; BLOCK_SIZE];
        let n = ring.lock().unwrap().pop(&mut out);
        for i in 0..n {
            assert_eq!(out[i], [received as i16, -(received as i16)]);
            received += 1;
        }
    }

    producer.join().unwrap();
}

#[test]
fn e08_15_constants() {
    assert!(RING_TARGET < RING_WAIT_ABOVE);
    assert!(RING_WAIT_ABOVE < RING_CAPACITY);
}
