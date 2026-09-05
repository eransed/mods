use std::{
  collections::HashMap,
  time::{self, Instant},
};

use apriltag::{Detector, Family, image_buf::DEFAULT_ALIGNMENT_U8};
use opencv::{
  core::{self, Point, Scalar, Size},
  highgui,
  imgcodecs::imencode,
  imgproc,
  prelude::*,
  videoio,
};

#[cfg(opencv4)]
use opencv::calib3d::{SOLVEPNP_IPPE_SQUARE, rodrigues, solve_pnp};

#[cfg(opencv5)]
use opencv::geometry::{SOLVEPNP_IPPE_SQUARE, rodrigues, solve_pnp};

use base64::prelude::*;
use opencv::core::{Point2f, Point3f, Vector};
use tokio::sync::{broadcast::Sender, watch::Receiver};
use tracing::{error, info, warn};
use types::{RawImageDetection, TagPose};

use crate::{message::Message, util::ValueWithStats};

#[derive(Clone, Copy, Debug)]
pub struct RPY {
  pub r: ValueWithStats<f64, 30>,
  pub p: ValueWithStats<f64, 30>,
  pub y: ValueWithStats<f64, 30>,
}

pub struct Frequency {
  start: Instant,
  count: u64,
}

impl Frequency {
  pub fn new() -> Self {
    Self { start: Instant::now(), count: 0 }
  }

  pub fn update(&mut self) -> f32 {
    let t = self.start.elapsed().as_secs_f32();
    self.count = self.count + 1;
    let f = self.count as f32 / t;
    f
  }
}

fn create_camera(device_index: i32, device_width: f64, use_gstreamer: bool) -> Option<videoio::VideoCapture> {
  if use_gstreamer {
    let pipeline =
      "libcamerasrc ! video/x-raw,width=1280,height=1080,format=BGR ! videoconvert ! appsink";
    let camera = videoio::VideoCapture::from_file(pipeline, videoio::CAP_GSTREAMER).expect("Failed to create gstreamer camera");
    return Some(camera);
  } else {
    let mut camera = match videoio::VideoCapture::new(device_index, videoio::CAP_ANY) {
      Ok(c) => c,
      Err(e) => {
        error!("Failed to create camera: {}", e);
        return None;
      }
    };

    match camera.set(videoio::CAP_PROP_FRAME_WIDTH, device_width) {
      Ok(_) => (),
      Err(e) => {
        error!("Failed to set CAP_PROP_FRAME_WIDTH: {}", e);
      }
    };

    match camera.get(videoio::CAP_PROP_FRAME_WIDTH) {
      Ok(w) => {
        info!("Camera CAP_PROP_FRAME_WIDTH: {}", w);
      }
      Err(e) => {
        error!("Failed to read camera CAP_PROP_FRAME_WIDTH: {}", e);
      }
    }
    return Some(camera);
  }
}

pub fn camera_start(
  sender: Sender<Message>,
  shutdown_rx: Receiver<bool>,
  config_rx: Receiver<types::Config>,
) {
  // Use the first configured camera as the active camera instance.
  let config = config_rx.borrow().clone();
  let Some(camera_config) = config.camera_configs.first() else {
    error!("No camera configuration is available");
    return;
  };
  let device_index = camera_config.device_index.value;
  let device_width = camera_config.device_width.value;
  let display = camera_config.opencv_display.value;
  let angle_filter = camera_config.angle_filter.value;
  let min_decision_margin = camera_config.min_decision_margin.value;
  let camera_fetch_delay_ms = camera_config.camera_fetch_delay_ms.value;
  let camera_send_image = camera_config.camera_send_image.value;
  let camera_send_image_resize_factor = camera_config.camera_send_image_resize_factor.value;
  let start = std::time::Instant::now();

  let window_title = "mods";

  info!("Trying to start camera: {} with frame width: {}", device_index, device_width);

  // Convert the configured camera index to the OpenCV integer type safely.
  let device_index = match i32::try_from(device_index) {
    Ok(index) => index,
    Err(_) => {
      error!("Camera device index is too large for OpenCV: {}", device_index);
      return;
    }
  };

  let mut camera = create_camera(device_index, device_width, false).unwrap();

  if !camera.is_opened().unwrap() {
    error!("Failed to open camera");
    return;
  } else {
    info!("Camera is open");
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

  let mut tag_rot_map: HashMap<usize, RPY> = Default::default();

  let filter_length = angle_filter as usize;

  let mut f = Frequency::new();

  loop {
    let cread_start = Instant::now();

    if *shutdown_rx.borrow() {
      info!("shutdown requested");
      break;
    }
    if config_rx.has_changed().unwrap_or(false) {
      info!("camera configuration changed; restarting camera");
      break;
    }

    camera.read(&mut frame).unwrap();

    if frame.empty() {
      warn!("Empty frame!");
      std::thread::sleep(time::Duration::from_millis(500));
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
      imgproc::cvt_color(&frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0).unwrap();
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

    let mut tags = vec![];

    for (_, det) in detections.iter().enumerate() {
      let pose_esti_start = Instant::now();
      // let id = det.id();
      // if id < 21 || id > 21 {
      //     continue;
      // }

      if det.decision_margin() < min_decision_margin {
        continue;
      }

      // Draw the tag outline and ID on the frame
      let corners = det.corners();
      for i in 0..4 {
        let p0 = Point::new(corners[i][0] as i32, corners[i][1] as i32);

        let p1 = Point::new(corners[(i + 1) % 4][0] as i32, corners[(i + 1) % 4][1] as i32);

        imgproc::line(
          &mut frame,
          p0,
          p1,
          Scalar::new(50.0, 255.0, 50.0, 0.0),
          2,
          imgproc::LINE_AA,
          0,
        )
        .unwrap();
      }

      // Draw the tag ID at the center of the tag
      let center = det.center();
      imgproc::put_text(
        &mut frame,
        &format!("{}", det.id()),
        Point::new(center[0] as i32, center[1] as i32),
        imgproc::FONT_HERSHEY_SIMPLEX,
        1.0,
        Scalar::new(50.0, 50.0, 255.0, 0.0),
        2,
        imgproc::LINE_AA,
        false,
      )
      .unwrap();

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
        Mat::from_slice_2d(&[&[0.0, 0.0, 0.0], &[0.0, 0.0, 0.0], &[0.0, 0.0, 0.0]]).unwrap();

      rodrigues(&rvec, &mut rotation_matrix, &mut core::no_array()).unwrap();

      // convert rotation matrix to Euler angles (roll, pitch, yaw)
      let (r, p, y) = convert_rotation_matrix_to_euler_angles(&rotation_matrix, true);

      // initialize the tag rotation map if it doesn't exist for this tag ID
      if tag_rot_map.get(&det.id()).is_none() {
        info!("Initializing rotation map for tag ID: {}", det.id());
        tag_rot_map.insert(
          det.id(),
          RPY { r: ValueWithStats::new(), p: ValueWithStats::new(), y: ValueWithStats::new() },
        );

        let k: Vec<&usize> = tag_rot_map.keys().collect();
        info!("Rotation map: {:?}", k);
      }

      tag_rot_map.get_mut(&det.id()).expect("Failed to get rotation map").r.push(r);
      tag_rot_map.get_mut(&det.id()).expect("Failed to get rotation map").p.push(p);
      tag_rot_map.get_mut(&det.id()).expect("Failed to get rotation map").y.push(y);

      // publish
      let tx = *tvec.at_2d::<f64>(0, 0).unwrap() * 10.0;
      let ty = *tvec.at_2d::<f64>(1, 0).unwrap() * 10.0;
      let tz = *tvec.at_2d::<f64>(2, 0).unwrap() * 10.0;

      let fr = tag_rot_map[&det.id()].r.mean_last_n(filter_length).unwrap_or_default();
      let fp = tag_rot_map[&det.id()].p.mean_last_n(filter_length).unwrap_or_default();
      let fy = tag_rot_map[&det.id()].y.mean_last_n(filter_length).unwrap_or_default();

      let tag_pose = TagPose {
        id: det.id(),
        center_image: (det.center()[0], det.center()[1]),
        decision_margin: det.decision_margin(),
        translation: (tx, ty, tz),
        rotation: (fr, fp, fy),
        pose_estimation_time_us: pose_esti_start.elapsed().as_micros() as u32,
      };

      tags.push(tag_pose);
    }

    let mut raw_image_detection = RawImageDetection {
      tags,
      image_data_base64: String::from(""),
      image_size: (0, 0),
      native_image_size: (frame.cols(), frame.rows()),
      detection_time_us: 0,
      image_encoding_time_us: 0,
      send_freq: 0.0,
    };

    let mut small_frame = Mat::default();
    if camera_send_image {
      let img_encoding_time = Instant::now();
      imgproc::resize(
        &frame,
        &mut small_frame,
        Size::default(),
        camera_send_image_resize_factor,
        camera_send_image_resize_factor,
        imgproc::INTER_AREA,
      )
      .unwrap();

      let par = core::Vector::<i32>::new();
      let mut png_encoded_frame = Vector::<u8>::new();

      imencode(".png", &small_frame, &mut png_encoded_frame, &par).unwrap();
      let base64_image = BASE64_STANDARD.encode(&png_encoded_frame);
      raw_image_detection.image_data_base64 = base64_image;
      raw_image_detection.image_size = (small_frame.cols(), small_frame.rows());
      raw_image_detection.image_encoding_time_us = img_encoding_time.elapsed().as_micros() as u32;
    }

    raw_image_detection.detection_time_us = cread_start.elapsed().as_micros() as u32;

    // f.update();
    raw_image_detection.send_freq = f.update();

    let m = Message::Broadcast {
      sender: "camera",
      body: serde_json::to_string(&raw_image_detection).unwrap(),
    };

    sender.send(m).unwrap();

    if display && camera_send_image {
      highgui::imshow(window_title, &small_frame).unwrap();
    }

    let key = highgui::wait_key(1).unwrap();

    if key >= 0 {
      let c = char::from_u32(key.try_into().unwrap());
      info!("key={} ({:?})", key, c);
      if key == ('q' as i32) {
        break;
      }
    }
    if camera_fetch_delay_ms > 0 {
      std::thread::sleep(std::time::Duration::from_millis(camera_fetch_delay_ms));
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
