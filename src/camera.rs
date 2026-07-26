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

use opencv::core::{Point2f, Point3f, Vector};
use tokio::sync::{broadcast::Sender, watch::Receiver};
use tracing::{info, warn};

use crate::{message::Message, util::ValueWithStats};

pub fn camera_start(
    sender: Sender<Message>,
    shutdown_rx: Receiver<bool>,
    device_index: i32,
    display: bool,
    skip_april_pose_estimation: bool,
    angle_filter: usize,
    min_decision_margin: f32,
) -> bool {
    let start = std::time::Instant::now();
    #[cfg(opencv4)]
    use opencv::calib3d::SOLVEPNP_IPPE_SQUARE;
    #[cfg(opencv4)]
    use opencv::calib3d::rodrigues;
    #[cfg(opencv4)]
    use opencv::calib3d::solve_pnp;

    #[cfg(opencv5)]
    use opencv::geometry::SOLVEPNP_IPPE_SQUARE;
    #[cfg(opencv5)]
    use opencv::geometry::rodrigues;
    #[cfg(opencv5)]
    use opencv::geometry::solve_pnp;

    let window_title = "mods";
    let mut res = false;
    let mut camera = videoio::VideoCapture::new(device_index, videoio::CAP_ANY).unwrap();
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

    const FILTER_LENGTH_CAP: usize = 60;
    let filter_length = angle_filter as usize;
    let mut roll = ValueWithStats::<f64, FILTER_LENGTH_CAP>::new();
    let mut pitch = ValueWithStats::<f64, FILTER_LENGTH_CAP>::new();
    let mut yaw = ValueWithStats::<f64, FILTER_LENGTH_CAP>::new();

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
        #[cfg(not(opencv_pre_411))]
        {
            imgproc::cvt_color(
                &frame,
                &mut gray,
                imgproc::COLOR_BGR2GRAY,
                0,
                core::AlgorithmHint::ALGO_HINT_DEFAULT,
            )
            .unwrap();
        }

        #[cfg(opencv_pre_411)]
        {
            imgproc::cvt_color(
                &frame,
                &mut gray,
                imgproc::COLOR_BGR2GRAY,
                0,
            )
            .unwrap();
        }

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

        let params = apriltag::TagParams {
            tagsize: 0.0225,
            fx: 1000 as f64,
            fy: 1000 as f64,
            cx: frame.cols() as f64 / 2.0,
            cy: frame.rows() as f64 / 2.0,
        };

        let detection_time = cread_start.elapsed();

        for (di, det) in detections.iter().enumerate() {
            let pose_esti_start = Instant::now();
            let id = det.id();
            // if id < 21 || id > 21 {
            //     continue;
            // }

            if det.decision_margin() < min_decision_margin {
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
                    Scalar::new(70.0, 255.0, 70.0, 0.0),
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
                Scalar::new(80.0, 80.0, 255.0, 0.0),
                2,
                imgproc::LINE_AA,
                false,
            )
            .unwrap();

            let rect = core::Rect {
                x: 1,
                y: 1,
                width: 700,
                height: 700,
            };

            let c = core::Scalar::new(0.0, 0.0, 0.0, 0.0);

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
                Scalar::new(255.0, 0.0, 255.0, 0.0),
                2,
                imgproc::LINE_AA,
                false,
            )
            .unwrap();

            if !skip_april_pose_estimation {
                // can segfault on apple silicon...
                // april start
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
            }
            // april stop

            // Build 3D object points for the tag corners (tag frame, Z=0 plane).
            // Order must match det.corners() order.
            let half_size = (params.tagsize / 2.0) as f32;
            let object_points = Vector::<Point3f>::from_slice(&[
                Point3f::new(-half_size, half_size, 0.0),
                Point3f::new(half_size, half_size, 0.0),
                Point3f::new(half_size, -half_size, 0.0),
                Point3f::new(-half_size, -half_size, 0.0),
            ]);

            let image_points = Vector::<Point2f>::from_slice(&[
                Point2f::new(corners[0][0] as f32, corners[0][1] as f32),
                Point2f::new(corners[1][0] as f32, corners[1][1] as f32),
                Point2f::new(corners[2][0] as f32, corners[2][1] as f32),
                Point2f::new(corners[3][0] as f32, corners[3][1] as f32),
            ]);

            let camera_matrix = Mat::from_slice_2d(&[
                &[params.fx, 0.0, params.cx],
                &[0.0, params.fy, params.cy],
                &[0.0, 0.0, 1.0],
            ])
            .unwrap();

            let dist_coeffs = Mat::default(); // assume no lens distortion

            // create a empty 3x3 matrix for rotation and a 3x1 matrix for translation
            let mut rvec = Mat::zeros(3, 1, core::CV_64F).unwrap().to_mat().unwrap();
            let mut tvec = Mat::zeros(3, 1, core::CV_64F).unwrap().to_mat().unwrap();

            solve_pnp(
                &object_points,
                &image_points,
                &camera_matrix,
                &dist_coeffs,
                &mut rvec,
                &mut tvec,
                false,
                SOLVEPNP_IPPE_SQUARE,
            )
            .unwrap();

            let mut rotation_matrix =
                Mat::from_slice_2d(&[&[0.0, 0.0, 0.0], &[0.0, 0.0, 0.0], &[0.0, 0.0, 0.0]])
                    .unwrap();

            rodrigues(&rvec, &mut rotation_matrix, &mut core::no_array()).unwrap();

            // Print translation (tvec is 3x1)
            for r in 0..3 {
                let ri32: i32 = r;
                // let t = *tvec.at::<f64>(r).unwrap();
                let t = *tvec.at_2d::<f64>(r, 0).unwrap();
                imgproc::put_text(
                    &mut frame,
                    &format!("{:.3}", t * 10.0),
                    Point::new(30 + 200 * ri32, 400),
                    imgproc::FONT_HERSHEY_SIMPLEX,
                    1.0,
                    Scalar::new(0.0, 255.0, 255.0, 0.0),
                    2,
                    imgproc::LINE_AA,
                    false,
                )
                .unwrap();
            }

            // Print rotation matrix (3x3)
            for r in 0..3 {
                for c in 0..3 {
                    let ri32: i32 = r;
                    let ci32: i32 = c;
                    let v = *rotation_matrix.at_2d::<f64>(r, c).unwrap();
                    imgproc::put_text(
                        &mut frame,
                        &format!("{:.2}", v),
                        Point::new(30 + 200 * ri32, 450 + 50 * ci32),
                        imgproc::FONT_HERSHEY_SIMPLEX,
                        1.0,
                        Scalar::new(255.0, 255.0, 0.0, 0.0),
                        2,
                        imgproc::LINE_AA,
                        false,
                    )
                    .unwrap();
                }
            }

            // convert rotation matrix to Euler angles (roll, pitch, yaw)
            let (r, p, y) = convert_rotation_matrix_to_euler_angles(&rotation_matrix, true);

            roll.push(r);
            pitch.push(p);
            yaw.push(y);

            imgproc::put_text(
                &mut frame,
                &format!(
                    "R: {:.2}, P: {:.2}, Y: {:.2}",
                    roll.mean_last_n(filter_length).unwrap_or_default(),
                    pitch.mean_last_n(filter_length).unwrap_or_default(),
                    yaw.mean_last_n(filter_length).unwrap_or_default()
                ),
                Point::new(50, 600),
                imgproc::FONT_HERSHEY_SIMPLEX,
                1.0,
                Scalar::new(255.0, 255.0, 255.0, 0.0),
                2,
                imgproc::LINE_AA,
                false,
            )
            .unwrap();

            // publish
            let tx = *tvec.at_2d::<f64>(0, 0).unwrap() * 10.0;
            let ty = *tvec.at_2d::<f64>(1, 0).unwrap() * 10.0;
            let tz = *tvec.at_2d::<f64>(2, 0).unwrap() * 10.0;
            let m = Message::Broadcast {
                sender: "cam",
                body: format!(
                    "id {} x: {:.3}, y: {:.3}, z: {:.3}, time: {:.1?}",
                    id,
                    tx,
                    ty,
                    tz,
                    pose_esti_start.elapsed()
                ),
            };
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
    if display {
        let _ = highgui::destroy_window(window_title).expect("Failed to destroy window");
        let _ = highgui::destroy_all_windows().expect("Failed to destroy all windows");
        highgui::wait_key(1).unwrap();
    }
    info!("Total runtime: {:.1?}", start.elapsed());
    return res;
}

fn convert_rotation_matrix_to_euler_angles(
    rotation_matrix: &Mat,
    use_degrees: bool,
) -> (f64, f64, f64) {
    let sy = (rotation_matrix.at_2d::<f64>(0, 0).unwrap().powi(2)
        + rotation_matrix.at_2d::<f64>(1, 0).unwrap().powi(2))
    .sqrt();
    let singular = sy < 1e-6;
    let (roll, pitch, yaw) = if singular {
        let roll = rotation_matrix
            .at_2d::<f64>(1, 2)
            .unwrap()
            .atan2(*rotation_matrix.at_2d::<f64>(1, 1).unwrap());
        let pitch = rotation_matrix.at_2d::<f64>(0, 2).unwrap().atan2(sy);
        let yaw: f64 = 0.0;
        (roll, pitch, yaw)
    } else {
        let roll = rotation_matrix
            .at_2d::<f64>(2, 1)
            .unwrap()
            .atan2(*rotation_matrix.at_2d::<f64>(2, 2).unwrap());
        let pitch = -rotation_matrix.at_2d::<f64>(2, 0).unwrap().atan2(sy);
        let yaw = rotation_matrix
            .at_2d::<f64>(1, 0)
            .unwrap()
            .atan2(*rotation_matrix.at_2d::<f64>(0, 0).unwrap());
        (roll, pitch, yaw)
    };

    if use_degrees {
        let roll = roll.to_degrees();
        let pitch = pitch.to_degrees();
        let yaw = yaw.to_degrees();
        (roll, pitch, yaw)
    } else {
        (roll, pitch, yaw)
    }
}
