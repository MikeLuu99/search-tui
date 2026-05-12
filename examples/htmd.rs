use htmd::HtmlToMarkdown;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::fs;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    let urls = vec![
        "https://www.rust-lang.org/",
        "https://www.wikipedia.org/",
        "https://news.ycombinator.com/",
        "https://www.bbc.com/",
        "https://www.espn.com/",
        "https://github.com/",
        "https://stackoverflow.com/",
        "https://www.nytimes.com/",
        "https://docs.python.org/3/",
        "https://www.reddit.com/",
    ];

    let wall_start = Instant::now();
    let combined = convert_html_to_md(&urls).await;
    let wall_ms = wall_start.elapsed().as_millis();

    fs::write("output.md", combined).await.unwrap();

    println!("---");
    println!("Total wall-clock time (concurrent): {wall_ms}ms");
    println!("Written to output.md");
}

async fn convert_html_to_md(urls: &[&str]) -> String {
    let client = Arc::new(reqwest::Client::new());
    let converter = Arc::new(Mutex::new(
        HtmlToMarkdown::builder()
            .skip_tags(vec!["script", "style"])
            .build(),
    ));
    let combined_md = Arc::new(Mutex::new(String::new()));

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for &url in urls {
        let client = Arc::clone(&client);
        let converter = Arc::clone(&converter);
        let combined = Arc::clone(&combined_md);
        let url = url.to_string();

        let handle = tokio::spawn(async move {
            // --- fetch ---
            let fetch_start = Instant::now();
            let html = match client.get(&url).send().await {
                Ok(resp) => match resp.text().await {
                    Ok(text) => text,
                    Err(e) => {
                        eprintln!("Body error for {url}: {e}");
                        return;
                    }
                },
                Err(e) => {
                    eprintln!("Fetch error for {url}: {e}");
                    return;
                }
            };
            let fetch_ms = fetch_start.elapsed().as_millis();

            // --- convert ---
            let convert_start = Instant::now();
            let md = {
                let conv = converter.lock().unwrap();
                conv.convert(&html).unwrap_or_default()
            };
            let convert_ms = convert_start.elapsed().as_millis();

            println!(
                "[{url}]\n  fetch: {fetch_ms}ms | convert: {convert_ms}ms | task total: {}ms",
                fetch_ms + convert_ms
            );

            // --- append to shared buffer ---
            let mut out = combined.lock().unwrap();
            out.push_str(&format!(
                "\n\n<!-- {url} | fetch: {fetch_ms}ms | convert: {convert_ms}ms -->\n"
            ));
            out.push_str(&md);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    Arc::try_unwrap(combined_md)
        .expect("Arc still has other owners")
        .into_inner()
        .expect("Mutex was poisoned")
}
