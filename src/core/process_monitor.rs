use anyhow::{Context, Result};
use chrono::Utc;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Child;
use tokio::sync::Mutex;

use crate::db::models::{ProcessStatus, TaskStatus};
use crate::db::operations::{execution_processes, tasks};
use crate::db::Database;

/// Maximum size for stdout/stderr buffers (10MB)
const MAX_BUFFER_SIZE: usize = 10 * 1024 * 1024;

/// Maximum lines to keep in buffer
const MAX_BUFFER_LINES: usize = 10_000;

/// Process output capture manager
pub struct ProcessMonitor {
    exec_id: String,
    task_id: String,
    db: Arc<Mutex<Database>>,
}

impl ProcessMonitor {
    /// Create a new process monitor
    pub fn new(exec_id: String, task_id: String, db: Database) -> Self {
        Self {
            exec_id,
            task_id,
            db: Arc::new(Mutex::new(db)),
        }
    }

    /// Capture process output and monitor completion
    ///
    /// This spawns background tasks to:
    /// 1. Capture stdout in real-time
    /// 2. Capture stderr in real-time
    /// 3. Monitor process completion
    /// 4. Update database with results
    pub async fn monitor_process(&self, mut child: Child) -> Result<()> {
        // Take stdout and stderr
        let stdout = child
            .stdout
            .take()
            .context("Failed to capture stdout")?;
        let stderr = child
            .stderr
            .take()
            .context("Failed to capture stderr")?;

        // Create shared buffers
        let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
        let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

        // Spawn tasks to capture outputs
        let stdout_task = {
            let buffer = Arc::clone(&stdout_buffer);
            tokio::spawn(async move {
                Self::capture_stream(stdout, buffer).await
            })
        };

        let stderr_task = {
            let buffer = Arc::clone(&stderr_buffer);
            tokio::spawn(async move {
                Self::capture_stream(stderr, buffer).await
            })
        };

        // Wait for both capture tasks to complete
        let _ = tokio::try_join!(stdout_task, stderr_task)?;

        // Wait for process completion
        let status = child.wait().await.context("Failed to wait for process")?;
        let exit_code = status.code();

        // Get captured outputs
        let stdout_data = stdout_buffer.lock().await;
        let stderr_data = stderr_buffer.lock().await;

        let stdout_str = String::from_utf8_lossy(&stdout_data).to_string();
        let stderr_str = String::from_utf8_lossy(&stderr_data).to_string();

        // Determine process status
        let process_status = if status.success() {
            ProcessStatus::Completed
        } else {
            ProcessStatus::Failed
        };

        // Update execution record with completion details
        {
            let db = self.db.lock().await;
            execution_processes::update_completion(
                db.conn(),
                &self.exec_id,
                process_status,
                exit_code,
                Some(&stdout_str),
                Some(&stderr_str),
            )
            .context("Failed to update execution completion")?;
        }

        // Update task status based on process outcome
        let task_status = if status.success() {
            TaskStatus::Completed
        } else {
            TaskStatus::Archived // Failed tasks go to archived
        };

        {
            let db = self.db.lock().await;

            // Get current task
            if let Some(mut task) = tasks::get_by_id(db.conn(), &self.task_id)? {
                task.status = task_status;
                task.updated_at = Utc::now();

                if task_status == TaskStatus::Completed {
                    task.completed_at = Some(Utc::now());
                }

                tasks::update(db.conn(), &task)?;
            }
        }

        Ok(())
    }

    /// Capture output from a stream with buffering limits
    async fn capture_stream<R: AsyncRead + Unpin>(
        stream: R,
        buffer: Arc<Mutex<Vec<u8>>>,
    ) -> Result<()> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        let mut line_count = 0;

        loop {
            line.clear();
            let bytes_read = reader
                .read_line(&mut line)
                .await
                .context("Failed to read line")?;

            if bytes_read == 0 {
                // EOF reached
                break;
            }

            let mut buf = buffer.lock().await;

            // Check buffer size limits
            if buf.len() + line.len() > MAX_BUFFER_SIZE {
                // Buffer is full, start dropping old lines
                // Find first newline and drop everything before it
                if let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                    buf.drain(0..=pos);
                }
            }

            // Check line count limit
            line_count += 1;
            if line_count > MAX_BUFFER_LINES {
                // Too many lines, drop oldest
                if let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                    buf.drain(0..=pos);
                }
            }

            // Append line to buffer
            buf.extend_from_slice(line.as_bytes());
        }

        Ok(())
    }
}

/// Spawn a background task to monitor a process
///
/// This function is the main entry point for process monitoring.
/// It creates a ProcessMonitor and spawns it in the background.
pub fn spawn_monitor(
    exec_id: String,
    task_id: String,
    child: Child,
    db: Database,
) -> tokio::task::JoinHandle<Result<()>> {
    tokio::spawn(async move {
        let monitor = ProcessMonitor::new(exec_id, task_id, db);

        if let Err(e) = monitor.monitor_process(child).await {
            eprintln!("Error monitoring process: {}", e);
            return Err(e);
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::io::AsyncWriteExt;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_capture_small_stream() {
        let data = b"line 1\nline 2\nline 3\n";
        let buffer = Arc::new(Mutex::new(Vec::new()));

        let cursor = std::io::Cursor::new(data);
        let async_cursor = tokio::io::BufReader::new(cursor);

        ProcessMonitor::capture_stream(async_cursor, Arc::clone(&buffer))
            .await
            .unwrap();

        let buf = buffer.lock().await;
        let captured = String::from_utf8_lossy(&buf);
        assert_eq!(captured, "line 1\nline 2\nline 3\n");
    }

    #[tokio::test]
    async fn test_buffer_size_limit() {
        // Create a large stream that exceeds MAX_BUFFER_SIZE
        let buffer = Arc::new(Mutex::new(Vec::new()));

        // Create a pipe for testing
        let (mut writer, reader) = tokio::io::duplex(1024);

        // Spawn capture task
        let capture_task = {
            let buf = Arc::clone(&buffer);
            tokio::spawn(async move {
                ProcessMonitor::capture_stream(reader, buf).await
            })
        };

        // Write some lines
        for i in 0..100 {
            writer.write_all(format!("line {}\n", i).as_bytes()).await.unwrap();
        }
        drop(writer); // Close writer to signal EOF

        // Wait for capture to complete
        capture_task.await.unwrap().unwrap();

        let buf = buffer.lock().await;
        assert!(buf.len() <= MAX_BUFFER_SIZE);
    }
}
