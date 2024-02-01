use crate::message::Message;

pub async fn handle_msg(msg: Message) {
    match msg {
        Message::Hello => {}
        //Message::Setup(setup) => match setup {
        //    SetupConfig::ModelSetting(s) => settings::set_model(s),
        //    SetupConfig::RestAPISetting(s) => settings::set_restapi(s),
        //    SetupConfig::IpCameraSetting {
        //        snapshot_url,
        //        stream_url,
        //    } => settings::set_camera(settings::CameraSetting::IpCamera {
        //        snapshot_url,
        //        stream_url,
        //    }),
        //    SetupConfig::LocalCameraSetting { camera_id } => {
        //        settings::set_camera(settings::CameraSetting::LocalCamera { camera_id })
        //    }
        //    SetupConfig::StepperXSetting(s) => settings::set_x_linear_stepper(s),
        //    SetupConfig::StepperYSetting(s) => settings::set_y_linear_stepper(s),
        //    SetupConfig::StepperZSetting(s) => settings::set_z_linear_stepper(s),
        //},
        //Message::ControlGoto(x, y, z) => {
        //    if let Some((x, y, z)) = goto(x, y, z).await {
        //        send_out(Message::ReportPosition(x, y, z))
        //    } else {
        //        send_out(Message::Error("IncompleteConfig".to_owned()))
        //    }
        //}
        //Message::ControlMove(x, y, z) => {
        //    if let Some((x, y, z)) = r#move(x, y, z).await {
        //        send_out(Message::ReportPosition(x, y, z))
        //    } else {
        //        send_out(Message::Error("IncompleteConfig".to_owned()))
        //    }
        //}
        _ => (),
    }
}
