/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Follows a child process stream line by line so stories await the line that
// signals readiness instead of polling a log file.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, Mutex};

// Third Party Imports
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::sync::mpsc;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Lines {
    received: mpsc::UnboundedReceiver<String>,
    transcript: Arc<Mutex<String>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Lines {
    pub fn follow(stream: impl AsyncRead + Unpin + Send + 'static) -> Self {
        let (sent, received) = mpsc::unbounded_channel();
        let transcript = Arc::new(Mutex::new(String::new()));
        let recorded = Arc::clone(&transcript);
        tokio::spawn(async move {
            let mut lines = BufReader::new(stream).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let mut transcript = recorded.lock().unwrap();
                transcript.push_str(&line);
                transcript.push('\n');
                drop(transcript);
                let _ = sent.send(line);
            }
        });
        Self {
            received,
            transcript,
        }
    }

    /// Awaits the next line that matches, failing once the stream closes.
    pub async fn until(&mut self, mut matches: impl FnMut(&str) -> bool) -> String {
        while let Some(line) = self.received.recv().await {
            if matches(&line) {
                return line;
            }
        }
        panic!(
            "the stream closed before the expected line:\n{}",
            self.transcript()
        );
    }

    pub async fn containing(&mut self, text: &str) -> String {
        self.until(|line| line.contains(text)).await
    }

    /// Awaits the end of the stream so the transcript is complete.
    pub async fn closed(&mut self) {
        while self.received.recv().await.is_some() {}
    }

    pub fn transcript(&self) -> String {
        self.transcript.lock().unwrap().clone()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
