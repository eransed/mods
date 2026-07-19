use apriltag::{Detector, Family, image_buf::DEFAULT_ALIGNMENT_U8};
use opencv::{
    core::{self, Point, Scalar},
    highgui, imgproc,
    prelude::*,
    videoio,
};

pub fn camera_start() -> opencv::Result<()> {
    let mut camera = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;

    if !camera.is_opened()? {
        panic!("Failed to open camera");
    }

    highgui::named_window("Camera", highgui::WINDOW_AUTOSIZE)?;

    // Create detector
    let builder = Detector::builder();
    let mut detector = builder
        .add_family_bits(Family::tag_16h5(), 1)
        .add_family_bits(Family::tag_36h11(), 1)
        .build()
        .expect("Failed to build a detector");

    let mut frame = Mat::default();
    let mut gray = Mat::default();

    loop {
        camera.read(&mut frame)?;

        if frame.empty() {
            continue;
        }

        // Convert to grayscale
        imgproc::cvt_color(
            &frame,
            &mut gray,
            imgproc::COLOR_BGR2GRAY,
            0,
            core::AlgorithmHint::ALGO_HINT_DEFAULT,
        )?;

        let mut image = apriltag::Image::zeros_with_alignment(
            gray.cols() as usize,
            gray.rows() as usize,
            DEFAULT_ALIGNMENT_U8
        ).expect("Failed to convert image");

        let src = gray.data_bytes()?;

        let width = gray.cols() as usize;
        let height = gray.rows() as usize;
        let src_stride = gray.step1(0)?; // bytes per row in the OpenCV image
        let dst_stride = image.stride();

        let dst = image.as_slice_mut();

        for y in 0..height {
            let src_row = &src[y * src_stride..y * src_stride + width];
            let dst_row = &mut dst[y * dst_stride..y * dst_stride + width];
            dst_row.copy_from_slice(src_row);
        }

        // let detections = detector.detect(&image)?;
        let detections = detector.detect(&image);

        for det in detections {
            // println!("Tag ID: {}", det.id());

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
                )?;
            }

            let center = det.center();

            imgproc::put_text(
                &mut frame,
                &format!("{}", det.id()),
                Point::new(center[0] as i32, center[1] as i32),
                imgproc::FONT_HERSHEY_SIMPLEX,
                0.6,
                Scalar::new(0.0, 0.0, 255.0, 0.0),
                2,
                imgproc::LINE_AA,
                false,
            )?;
        }

        highgui::imshow("Camera", &frame)?;

        if highgui::wait_key(1)? >= 0 {
            break;
        }
    }

    let _ = highgui::destroy_all_windows();

    Ok(())
}
