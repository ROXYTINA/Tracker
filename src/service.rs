use crate::error::AppError;
use service_manager::*;
use std::ffi::OsString;

pub fn install_service() -> Result<(), AppError> {
    let manager = <dyn ServiceManager>::native()
        .map_err(|e| AppError::Other(format!("Failed to get service manager: {}", e)))?;

    let exe_path = std::env::current_exe()
        .map_err(|e| AppError::Io(e))?;

    manager.install(ServiceInstallCtx {
        label: "tracker".parse().unwrap(),
        program: exe_path,
        args: vec![OsString::from("daemon")],
        contents: None,
        username: None,
        working_directory: None,
        environment: None,
        autostart: true,
        restart_policy: RestartPolicy::Always { delay_secs: Some(5) },
    }).map_err(|e| AppError::Other(format!("Failed to install service: {}", e)))?;

    #[cfg(windows)]
    {
        use std::process::Command;
        let _ = Command::new("powershell")
            .args(&[
                "-Command",
                "New-Item -Path 'HKCU:\\Software\\Classes\\AppUserModelId\\com.kanha.tracker' -Force; New-ItemProperty -Path 'HKCU:\\Software\\Classes\\AppUserModelId\\com.kanha.tracker' -Name 'DisplayName' -Value 'Tracker' -PropertyType String -Force"
            ])
            .output();
    }

    Ok(())
}

pub fn uninstall_service() -> Result<(), AppError> {
    let manager = <dyn ServiceManager>::native()
        .map_err(|e| AppError::Other(format!("Failed to get service manager: {}", e)))?;

    manager.uninstall(ServiceUninstallCtx {
        label: "tracker".parse().unwrap(),
    }).map_err(|e| AppError::Other(format!("Failed to uninstall service: {}", e)))?;

    Ok(())
}

pub fn start_service() -> Result<(), AppError> {
    let manager = <dyn ServiceManager>::native()
        .map_err(|e| AppError::Other(format!("Failed to get service manager: {}", e)))?;

    manager.start(ServiceStartCtx {
        label: "tracker".parse().unwrap(),
    }).map_err(|e| AppError::Other(format!("Failed to start service: {}", e)))?;

    Ok(())
}

pub fn stop_service() -> Result<(), AppError> {
    let manager = <dyn ServiceManager>::native()
        .map_err(|e| AppError::Other(format!("Failed to get service manager: {}", e)))?;

    manager.stop(ServiceStopCtx {
        label: "tracker".parse().unwrap(),
    }).map_err(|e| AppError::Other(format!("Failed to stop service: {}", e)))?;

    Ok(())
}
