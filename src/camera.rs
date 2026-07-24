use std::time::Instant;

use apriltag::{Detector, Family, image_buf::DEFAULT_ALIGNMENT_U8};
use opencv::{
    core::{self, Point, Scalar, Size},
    highgui,
    imgproc::{
        self,
        LineTypes::{FILLED, LINE_AA},
    },
    prelude::*,
    videoio,
};
use tokio::sync::{broadcast::Sender, watch::Receiver};
use tracing::{info, warn};

use crate::message::Message;

pub fn camera_start(sender: Sender<Message>, shutdown_rx: Receiver<bool>, display: bool) -> bool {
    let start = std::time::Instant::now();
    let window_title = "mods";

    let mut res = false;
    let mut camera = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();
    camera
        .set(videoio::CAP_PROP_FRAME_WIDTH, 1920 as f64)
        .unwrap();

    if !camera.is_opened().unwrap() {
        panic!("Failed to open camera");
    }

    if display {
        highgui::named_window(window_title, highgui::WINDOW_AUTOSIZE).unwrap();
    }

    let builder = Detector::builder();
    let mut detector = builder
        .add_family_bits(Family::tag_16h5(), 1)
        // .add_family_bits(Family::tag_36h11(), 1)
        .build()
        .expect("Failed to build a detector");

    let mut frame = Mat::default();
    let mut gray = Mat::default();

    let mut first_frame = false;

    loop {
        let cread_start = Instant::now();

        if *shutdown_rx.borrow() {
            info!("shutdown requested");
            break;
        }

        camera.read(&mut frame).unwrap();

        let processing_start = Instant::now();
        if frame.empty() {
            warn!("Empty frame!");
            continue;
        }

        if !first_frame {
            first_frame = true;
            let size = frame.size().unwrap();
            info!("Frame size: {:?}", size);
            info!("Camera startup time: {:.1?}", start.elapsed());
        }

        // Convert to grayscale
        imgproc::cvt_color(
            &frame,
            &mut gray,
            imgproc::COLOR_BGR2GRAY,
            0,
            core::AlgorithmHint::ALGO_HINT_DEFAULT,
        )
        .unwrap();

        // convert to image that the apriltag lib understands
        // todo: optimize
        let mut image = apriltag::Image::zeros_with_alignment(
            gray.cols() as usize,
            gray.rows() as usize,
            DEFAULT_ALIGNMENT_U8,
        )
        .expect("Failed to convert image");

        let src = gray.data_bytes().unwrap();

        let width = gray.cols() as usize;
        let height = gray.rows() as usize;

        // bytes per row in the OpenCV image
        let src_stride = gray.step1(0).unwrap();
        let dst_stride = image.stride();

        let dst = image.as_slice_mut();

        for y in 0..height {
            let src_row = &src[y * src_stride..y * src_stride + width];
            let dst_row = &mut dst[y * dst_stride..y * dst_stride + width];
            dst_row.copy_from_slice(src_row);
        }

        let detections = detector.detect(&image);

        // if detections.len() < 1 {
        //     std::thread::sleep(Duration::from_millis(1000));
        //     continue;
        // }

        let params = apriltag::TagParams {
            tagsize: 0.0225,
            fx: 200000 as f64,
            fy: 200000 as f64,
            cx: 960 as f64,
            cy: 540 as f64,
        };

        let detection_time = cread_start.elapsed();

        for (di, det) in detections.iter().enumerate() {
            let id = det.id();
            // if id < 21 || id > 21 {
            //     continue;
            // }

            if det.decision_margin() < 40.0 {
                continue;
            }

            // only show info about the first detection
            // if di > 0 {
            //     continue;
            // }

            let corners = det.corners();

            for i in 0..4 {
                let p0 = Point::new(corners[i][0] as i32, corners[i][1] as i32);

                let p1 = Point::new(
                    corners[(i + 1) % 4][0] as i32,
                    corners[(i + 1) % 4][1] as i32,
                );

                imgproc::line(
                    &mut frame,
                    p0,
                    p1,
                    Scalar::new(0.0, 255.0, 0.0, 0.0),
                    2,
                    imgproc::LINE_AA,
                    0,
                )
                .unwrap();
            }

            let center = det.center();

            imgproc::put_text(
                &mut frame,
                &format!("{}. T{} ({:.1})", di, det.id(), det.decision_margin()),
                Point::new(center[0] as i32, center[1] as i32),
                imgproc::FONT_HERSHEY_SIMPLEX,
                1.0,
                Scalar::new(0.0, 0.0, 255.0, 0.0),
                2,
                imgproc::LINE_AA,
                false,
            )
            .unwrap();

            let rect = core::Rect {
                x: 10,
                y: 10,
                width: 700,
                height: 400,
            };

            let c = core::Scalar::new(0.0, 0.0, 0.0, 50.0);

            imgproc::rectangle(&mut frame, rect, c, FILLED.into(), LINE_AA.into(), 0).unwrap();

            imgproc::put_text(
                &mut frame,
                &format!(
                    "Detection time: {:.1?} - PT: {:.1?}",
                    detection_time,
                    processing_start.elapsed()
                ),
                Point::new(30, 50),
                imgproc::FONT_HERSHEY_SIMPLEX,
                1.0,
                Scalar::new(255.0, 255.0, 255.0, 0.0),
                2,
                imgproc::LINE_AA,
                false,
            )
            .unwrap();

            // can segfault on apple silicon...
            let pe = apriltag::Detection::estimate_tag_pose(&det, &params).unwrap();
            let tra = pe.translation().data();
            let mut index = 0;
            for r in 0..pe.translation().nrows() {
                for c in 0..pe.translation().ncols() {
                    let ri32: i32 = r.try_into().unwrap();
                    let ci32: i32 = c.try_into().unwrap();
                    imgproc::put_text(
                        &mut frame,
                        &format!("{:.3}", tra[index] * 10000 as f64),
                        Point::new(30 + 200 * ri32, 100 + 50 * ci32),
                        imgproc::FONT_HERSHEY_SIMPLEX,
                        1.0,
                        Scalar::new(255.0, 255.0, 255.0, 0.0),
                        2,
                        imgproc::LINE_AA,
                        false,
                    )
                    .unwrap();
                    index = index + 1;
                }
            }

            let rot = pe.rotation().data();
            let mut index = 0;
            for r in 0..pe.rotation().nrows() {
                for c in 0..pe.rotation().ncols() {
                    let ri32: i32 = r.try_into().unwrap();
                    let ci32: i32 = c.try_into().unwrap();
                    imgproc::put_text(
                        &mut frame,
                        &format!("{:.2}", rot[index]),
                        Point::new(30 + 200 * ri32, 150 + 50 * ci32),
                        imgproc::FONT_HERSHEY_SIMPLEX,
                        1.0,
                        Scalar::new(10.0, 255.0, 10.0, 0.0),
                        2,
                        imgproc::LINE_AA,
                        false,
                    )
                    .unwrap();
                    index = index + 1;
                }
            }
            // publish

            let m = Message::Broadcast { sender: "cam", body: format!("id {} x: {:.3}, y: {:.3}, z: {:.3}", id, tra[0], tra[1], tra[2]) };
            sender.send(m).unwrap();
        }

        let mut small_frame = Mat::default();
        let resize_factor = 0.4;
        imgproc::resize(
            &frame,
            &mut small_frame,
            Size::default(),
            resize_factor,
            resize_factor,
            imgproc::INTER_AREA,
        )
        .unwrap();

        if display {
            highgui::imshow(window_title, &small_frame).unwrap();
        }

        let key = highgui::wait_key(1).unwrap();

        if key >= 0 {
            let c = char::from_u32(key.try_into().unwrap());
            info!("key={} ({:?})", key, c);
            if key == ('q' as i32) {
                res = true;
                break;
            }
        }
    }

    info!("Shutting down...");
    let _ = camera.release().expect("Failed to release camera");
    let _ = highgui::destroy_window(window_title).expect("Failed to destroy window");
    let _ = highgui::destroy_all_windows().expect("Failed to destroy all windows");
    highgui::wait_key(1).unwrap();
    info!("Total runtime: {:.1?}", start.elapsed());
    return res;
}
