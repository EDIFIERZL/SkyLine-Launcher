use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;

pub struct SegmentedDownloader {
    client: reqwest::Client,
    concurrency: usize,
    segment_size: u64,
    min_size_for_segment: u64,
}

impl SegmentedDownloader {
    pub fn new(concurrency: usize) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(crate::mc::mirror::SKYLINE_USER_AGENT)
                .connect_timeout(std::time::Duration::from_secs(15))
                .timeout(std::time::Duration::from_secs(180))
                .tcp_keepalive(Some(std::time::Duration::from_secs(30)))
                .build()
                .unwrap(),
            concurrency,
            segment_size: 4 * 1024 * 1024, 
            min_size_for_segment: 1024 * 1024, 
        }
    }

    pub fn with_segment_size(mut self, size: u64) -> Self {
        self.segment_size = size;
        self
    }

    pub async fn download_many(
        &self,
        urls: Vec<(String, PathBuf)>,
        on_progress: impl Fn(u64, u64) + Send + Sync + 'static,
    ) -> Result<(), String> {
        let total = urls.len() as u64;
        let done = Arc::new(AtomicU64::new(0));
        let semaphore = Arc::new(Semaphore::new(self.concurrency));
        let client = Arc::new(self.client.clone());
        let on_progress = Arc::new(on_progress);

        for (url, path) in urls {
            let sem = semaphore.clone();
            let cl = client.clone();
            let d = done.clone();
            let p = on_progress.clone();

            tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                match cl.get(&url).send().await {
                    Ok(resp) => {
                        if let Ok(bytes) = resp.bytes().await {
                            let _ = std::fs::write(&path, &bytes);
                        }
                    }
                    Err(e) => log::warn!("Download failed: {} - {}", url, e),
                }
                let current = d.fetch_add(1, Ordering::Relaxed) + 1;
                (p)(current, total);
            });
        }
        Ok(())
    }

    pub async fn download_segmented(
        &self,
        url: &str,
        path: &PathBuf,
        on_progress: Option<Box<dyn Fn(u64, u64) + Send>>,
    ) -> Result<u64, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let hread_resp = self.client
            .head(url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let total_size = hread_resp.content_length().unwrap_or(0);
        let supports_range = hread_resp
            .headers()
            .get("accept-ranges")
            .map(|v| v.to_str().unwrap_or("").contains("bytes"))
            .unwrap_or(false);

        if total_size < self.min_size_for_segment || !supports_range {
            return self.download_simple(url, path, on_progress).await;
        }

        let segment_count = (total_size + self.segment_size - 1) / self.segment_size;
        let semaphore = Arc::new(Semaphore::new(self.concurrency));
        let downloaderd = Arc::new(AtomicU64::new(0));
        let client = Arc::new(self.client.clone());

        {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)
                .map_err(|e| e.to_string())?;
            file.set_len(total_size).map_err(|e| e.to_string())?;
        }

        let mut handles = Vec::new();

        for i in 0..segment_count {
            let start = i * self.segment_size;
            let end = std::cmp::min(start + self.segment_size - 1, total_size - 1);
            let sem = semaphore.clone();
            let cl = client.clone();
            let dl = downloaderd.clone();
            let url = url.to_string();
            let path = path.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                let resp = cl
                    .get(&url)
                    .header("Range", format!("bytes={}-{}", start, end))
                    .send()
                    .await
                    .map_err(|e| format!("Segment {} failed: {}", i, e))?;

                let bytes = resp.bytes().await
                    .map_err(|e| format!("Segment {} read failed: {}", i, e))?;

                let mut file = tokio::fs::OpenOptions::new()
                    .write(true)
                    .open(&path)
                    .await
                    .map_err(|e| e.to_string())?;

                use tokio::io::AsyncSeekExt;
                file.seek(std::io::SeekFrom::Start(start)).await
                    .map_err(|e| e.to_string())?;
                file.write_all(&bytes).await
                    .map_err(|e| e.to_string())?;

                dl.fetch_add(bytes.len() as u64, Ordering::Relaxed);
                Ok::<(), String>(())
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.await.map_err(|e| e.to_string())??;
        }

        if let Some(ref callback) = on_progress {
            callback(total_size, total_size);
        }

        Ok(total_size)
    }

    async fn download_simple(
        &self,
        url: &str,
        path: &PathBuf,
        on_progress: Option<Box<dyn Fn(u64, u64) + Send>>,
    ) -> Result<u64, String> {
        let resp = self.client.get(url).send().await.map_err(|e| e.to_string())?;
        let total_size = resp.content_length().unwrap_or(0);

        let mut stream = resp.bytes_stream();
        let mut file = tokio::fs::File::create(path).await.map_err(|e| e.to_string())?;
        let mut downloaderd: u64 = 0;

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| e.to_string())?;
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;
            downloaderd += chunk.len() as u64;

            if let Some(ref callback) = on_progress {
                callback(downloaderd, total_size);
            }
        }

        file.flush().await.map_err(|e| e.to_string())?;
        Ok(downloaderd)
    }
}

pub struct MultiDownloader {
    inner: SegmentedDownloader,
}

impl MultiDownloader {
    pub fn new(concurrency: usize) -> Self {
        Self {
            inner: SegmentedDownloader::new(concurrency),
        }
    }

    pub async fn download_many(
        &self,
        urls: Vec<(String, PathBuf)>,
        on_progress: impl Fn(u64, u64) + Send + Sync + 'static,
    ) -> Result<(), String> {
        self.inner.download_many(urls, on_progress).await
    }
}
