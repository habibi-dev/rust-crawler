use crate::features::crawler::Browser;
use crate::features::sites::model::site::Model;
use crate::features::sites::repository::post_repository::PostRepository;
use crate::features::sites::repository::site_repository::SiteRepository;
use crate::features::sites::utility::normalize_link::normalize_link;
use crate::features::sites::validation::post_form::PostFormCreate;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub async fn check_new_post() {
    let sites = match SiteRepository::all().await {
        Ok(list) => list,
        Err(e) => {
            eprintln!("Failed to load sites: {e}");
            return;
        }
    };

    // limit concurrency to 5 sites at a time
    let sem = Arc::new(Semaphore::new(5));

    for site in sites {
        let sem = sem.clone();

        let permit = match sem.acquire_owned().await {
            Ok(p) => p,
            Err(_) => break,
        };

        tokio::spawn(async move {
            // keep permit alive for whole site processing
            let _permit = permit;

            if let Err(e) = process_site(site).await {
                eprintln!("process_site failed: {e}");
            }
        });
    }
}

async fn process_site(site: Model) -> anyhow::Result<()> {
    let path = match &site.path_link {
        Some(p) if !p.is_empty() => p,
        _ => return Ok(()),
    };

    let browser = match Browser::new(&site.url_list, None, None).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Browser failed to start: {}", e);
            return Ok(());
        }
    };

    if let Some(remove_str) = &site.path_remove {
        let selectors: Vec<String> = remove_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if !selectors.is_empty()
            && let Err(e) = browser.remove_elements(selectors).await
        {
            eprintln!("Failed to remove elements for site {}: {}", site.id, e);
        }
    }

    let links = browser
        .get_attrs(path, "href")
        .await
        .map_err(|e| anyhow::anyhow!("get_attrs failed for site {}: {}", site.id, e))?;

    for raw_link in links {
        let link = normalize_link(&site.url, &raw_link);

        if let Err(e) = PostRepository::create(PostFormCreate {
            url: Some(link),
            site_id: site.id,
            user_id: site.user_id,
            api_key_id: site.api_key_id,
        })
        .await
        {
            // ignore unique constraint errors
            let msg = e.to_string();
            if !msg.contains("UNIQUE constraint failed") {
                eprintln!("Failed to create post for site {}: {}", site.id, msg);
            }
        }
    }

    Ok(())
}
