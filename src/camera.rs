use opencv::{core, highgui, prelude::*, videoio};

pub fn camera_start() -> opencv::Result<()> {
    let mut camera = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;

    if !camera.is_opened()? {
        panic!("Failed to open camera");
    }

    highgui::named_window("Camera", highgui::WINDOW_AUTOSIZE)?;

    let mut frame = core::Mat::default();

    loop {
        camera.read(&mut frame)?;

        if frame.empty() {
            continue;
        }

        highgui::imshow("Camera", &frame)?;

        if highgui::wait_key(1)? >= 0 {
            break;
        }
    }

    Ok(())
}
