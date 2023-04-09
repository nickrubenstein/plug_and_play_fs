use std::{
    path::MAIN_SEPARATOR,
    process::Command,
    sync::mpsc::{self, Sender, TryRecvError},
    thread,
    time::Duration, str::from_utf8,
};

use crate::models::folder::Folder;

#[derive(Debug)]
pub struct TimelapseThread {
    thread: Option<thread::JoinHandle<()>>,
    tx: Option<Sender<()>>,
}

impl TimelapseThread {
    pub fn new() -> Self {
        Self {
            thread: None,
            tx: None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.thread.is_some() && !self.thread.as_ref().unwrap().is_finished()
    }

    pub fn start(
        &mut self,
        frequency: u64,
        quality: u64,
        folder_name: String,
        file_prefix: String,
    ) {
        let (tx, rx) = mpsc::channel::<()>();
        self.tx = Some(tx);
        let mut count: u64 = 1;
        self.thread = Some(thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(frequency));
            match rx.try_recv() {
                Ok(()) => break,
                Err(err) if err == TryRecvError::Empty => {
                    match Command::new("sh")
                      .arg("-c")
                      // .arg("raspistill")
                      // .arg(format!("-q={}", quality.to_string()))
                      // .arg(format!(
                      //         "-o=\".{}{}{}{}{}-{}.jpg\"",
                      //         MAIN_SEPARATOR,
                      //         Folder::root_folder(),
                      //         folder_name,
                      //         MAIN_SEPARATOR,
                      //         file_prefix,
                      //         count)
                      // )
                      .arg("echo hello")
                      .output()
                    {
                        Ok(output) => {
                            println!("output.status = {}", output.status);
                            println!("output.stderr = {:?}", from_utf8(&output.stderr));
                            println!("output.stdout = {:?}", from_utf8(&output.stdout));
                        }
                        Err(err) => {
                            println!("Timelapse command output err: {}", err);
                        }
                    };
                    count += 1;
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
            }
            None => {}
        };
    }
}
