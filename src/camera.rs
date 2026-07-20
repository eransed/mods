use apriltag::{Detector, Family, image_buf::DEFAULT_ALIGNMENT_U8};
use opencv::{
    core::{self, Point, Scalar, Size},
    highgui, imgproc,
    prelude::*,
    videoio,
};
use tracing::{info, warn};

pub fn camera_start() -> bool {
    let window_tile = "mods";

    let mut res = false;
    let mut camera = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();
    camera
        .set(videoio::CAP_PROP_FRAME_WIDTH, 1920 as f64)
        .unwrap();

    if !camera.is_opened().unwrap() {
        panic!("Failed to open camera");
    }

    highgui::named_window(window_tile, highgui::WINDOW_AUTOSIZE).unwrap();

    // Create detector
    let builder = Detector::builder();
    let mut detector = builder
        .add_family_bits(Family::tag_16h5(), 1)
        .add_family_bits(Family::tag_36h11(), 1)
        .build()
        .expect("Failed to build a detector");

    let mut frame = Mat::default();
    let mut gray = Mat::default();

    let mut first_frame = false;

    loop {
        camera.read(&mut frame).unwrap();

        if frame.empty() {
            warn!("Empty frame!");
            continue;
        }

        if !first_frame {
            first_frame = true;
            let size = frame.size().unwrap();
            info!("Frame size: {:?}", size);
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

        let mut image = apriltag::Image::zeros_with_alignment(
            gray.cols() as usize,
            gray.rows() as usize,
            DEFAULT_ALIGNMENT_U8,
        )
        .expect("Failed to convert image");

        let src = gray.data_bytes().unwrap();

        let width = gray.cols() as usize;
        let height = gray.rows() as usize;
        let src_stride = gray.step1(0).unwrap(); // bytes per row in the OpenCV image
        let dst_stride = image.stride();

        let dst = image.as_slice_mut();

        for y in 0..height {
            let src_row = &src[y * src_stride..y * src_stride + width];
            let dst_row = &mut dst[y * dst_stride..y * dst_stride + width];
            dst_row.copy_from_slice(src_row);
        }

        // let detections = detector.detect(&image)?;
        let detections = detector.detect(&image);

        let params = apriltag::TagParams {
            tagsize: 0.0225,
            fx: 200 as f64,
            fy: 200 as f64,
            cx: 780 as f64,
            cy: 438 as f64,
        };

        for det in detections {
            // println!("Tag ID: {}", det.id());
            if det.id() != 21 {
                continue;
            }

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
                &format!("{}", det.id()),
                Point::new(center[0] as i32, center[1] as i32),
                imgproc::FONT_HERSHEY_SIMPLEX,
                0.9,
                Scalar::new(0.0, 0.0, 255.0, 0.0),
                2,
                imgproc::LINE_AA,
                false,
            )
            .unwrap();

            let pe = apriltag::Detection::estimate_tag_pose(&det, &params).unwrap();
            let rot = pe.rotation().data();
            let mut index = 0;

            for r in 0..pe.rotation().nrows() {
                for c in 0..pe.rotation().ncols() {
                    let ri32: i32 = r.try_into().unwrap();
                    let ci32: i32 = c.try_into().unwrap();
                    imgproc::put_text(
                        &mut frame,
                        &format!("{:.2}", rot[index]),
                        Point::new(30 + 200*ri32, 50 + 50*ci32),
                        imgproc::FONT_HERSHEY_SIMPLEX,
                        0.9,
                        Scalar::new(255.0, 255.0, 255.0, 0.0),
                        2,
                        imgproc::LINE_AA,
                        false,
                    )
                    .unwrap();
                    index = index + 1;
                }
            }
        }

        let mut small_frame = Mat::default();
        imgproc::resize(
            &frame,
            &mut small_frame,
            Size::default(),
            0.5,
            0.5,
            imgproc::INTER_AREA,
        )
        .unwrap();

        highgui::imshow(window_tile, &small_frame).unwrap();

        let key = highgui::wait_key(1).unwrap();

        if key >= 0 {
            let c = char::from_u32(key.try_into().unwrap());
            info!("key={} ({:?})", key, c);
            if key == ('q' as i32) {
                res = true;
            }
            break;
        }
    }

    let _ = highgui::destroy_all_windows();

    return res;
}
