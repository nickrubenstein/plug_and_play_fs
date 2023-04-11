use std::{
    path::MAIN_SEPARATOR,
    process::Command,
    sync::mpsc::{self, Sender, TryRecvError},
    thread,
    time::Duration, str::from_utf8,
};

use serde::Deserialize;

use crate::models::folder::Folder;

use super::time_format;

#[derive(Deserialize, Debug)]
pub struct TimelapseSettings {
  frequency: u64,
  quality: u64,
  folder_name: String,
  file_prefix: String
}

impl Clone for TimelapseSettings {
    fn clone(&self) -> Self {
        Self { frequency: self.frequency.clone(), quality: self.quality.clone(), folder_name: self.folder_name.clone(), file_prefix: self.file_prefix.clone() }
    }
}

#[derive(Debug)]
pub struct TimelapseThread {
    thread: Option<thread::JoinHandle<()>>,
    tx: Option<Sender<()>>,
    settings: Option<TimelapseSettings>
}

impl TimelapseThread {
    pub fn new() -> Self {
        Self {
            thread: None,
            tx: None,
            settings: None
        }
    }

    pub fn is_running(&self) -> bool {
        self.thread.is_some() && !self.thread.as_ref().unwrap().is_finished()
    }

    pub fn start(&mut self, settings: TimelapseSettings) {
        self.settings = Some(settings.to_owned());
        let (tx, rx) = mpsc::channel::<()>();
        self.tx = Some(tx);
        self.thread = Some(thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(settings.frequency));
            match rx.try_recv() {
                Ok(()) => break,
                Err(err) if err == TryRecvError::Empty => {
                    let time = time_format::now(Some("[year]_[month]_[day]-[hour]_[minute]_[second]"));
                    match Command::new("raspistill")
                      .arg("-t")
                      .arg("1000")
                      .arg("-q")
                      .arg(format!("{}", settings.quality.to_string()))
                      .arg("-o")
                      .arg(format!(
                              "{}{}{}{}-{}.jpg",
                              Folder::root_folder(),
                              settings.folder_name,
                              MAIN_SEPARATOR,
                              settings.file_prefix,
                              time)
                      )
                      .output()
                    {
                        Ok(output) => {
                          if !output.status.success() {
                            println!("Timelapse output.status = {}", output.status);
                            println!("Timelapse output.stderr = {:?}", from_utf8(&output.stderr));
                            println!("Timelapse output.stdout = {:?}", from_utf8(&output.stdout));
                          }
                        }
                        Err(err) => {
                            println!("Timelapse command output err: {}", err);
                        }
                    };
                }
                Err(err) => {
                    println!("Timelapse try receive err: {}", err);
                }
            }
        }));
    }

    pub fn stop(&mut self) {
        match &self.tx {
            Some(tx) => {
                match tx.send(()) {
                    Ok(()) => {}
                    Err(err) => {
                        println!("Timelapse stop err: {}", err);
                    }
                };
                self.tx = None;
                self.thread = None;
                self.settings = None;
            }
            None => {}
        };
    }
}
