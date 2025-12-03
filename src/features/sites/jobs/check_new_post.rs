use crate::features::crawler::Browser;
use crate::features::sites::model::site::Model;
use crate::features::sites::repository::post_repository::PostRepository;
use crate::features::sites::repository::site_repository::SiteRepository;
use crate::features::sites::utility::normalize_link::normalize_link;
use crate::features::sites::utility::site_error_tracker::{register_site_error, reset_site_error};
use crate::features::sites::validation::post_form::PostFormCreate;
use tokio::time::{Duration, sleep, timeout};

pub async fn check_new_post() {
    let sites = match SiteRepository::all().await {
        Ok(list) => list,
        Err(e) => {
            eprintln!("Failed to load sites: {e}");
            return;
        }
    };

    let mut is_first_site = true;

    for site in sites {
        if !is_first_site {
            // Delay between site visits to avoid aggressive crawling behavior
            sleep(Duration::from_secs(3)).await;
        }

        is_first_site = false;

        let timeout_result =
            tokio::time::timeout(Duration::from_secs(60), process_site(site)).await;

        match timeout_result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => eprintln!("process_site failed: {e}"),
            Err(_) => eprintln!("process_site timeout"),
        }
    }
}

async fn process_site(site: Model) -> anyhow::Result<()> {
    let path = match &site.path_link {
        Some(p) if !p.is_empty() => p,
        _ => return Ok(()),
    };

    // Timeout for Browser::new
    let browser = match timeout(
        Duration::from_secs(30),
        Browser::new(&site.url_list, None, None),
    )
    .await
    {
        Ok(Ok(b)) => b,
        Ok(Err(e)) => {
            eprintln!("Browser failed to start for site {}: {}", site.id, e);

            block(&site).await;

            return Ok(());
        }
        Err(_) => {
            eprintln!("Browser startup timeout for site {}", site.id);
            block(&site).await;
            return Ok(());
        }
    };

    if let Err(e) = browser
        .wait_for_selector(path, Duration::from_secs(20))
        .await
    {
        eprintln!(
            "wait_for_selector failed for site {} (selector: {}): {}",
            site.id, path, e
        );

        block(&site).await;

        return Ok(());
    }

    reset_site_error(site.id).await;

    if let Some(remove_str) = &site.path_remove {
        let selectors: Vec<String> = remove_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if !selectors.is_empty() {
            let remove_result =
                timeout(Duration::from_secs(10), browser.remove_elements(selectors)).await;

            match remove_result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    eprintln!("Failed to remove elements for site {}: {}", site.id, e);
                }
                Err(_) => {
                    eprintln!("remove_elements timeout for site {}", site.id);
                    block(&site).await;
                }
            }
        }
    }

    let links = match timeout(Duration::from_secs(20), browser.get_attrs(path, "href")).await {
        Ok(Ok(links)) => links,
        Ok(Err(e)) => {
            return Err(anyhow::anyhow!(
                "get_attrs failed for site {}: {}",
                site.id,
                e
            ));
        }
        Err(_) => {
            eprintln!("get_attrs timeout for site {}", site.id);
            block(&site).await;
            return Ok(());
        }
    };

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
            let msg = e.to_string();
            if !msg.contains("UNIQUE constraint failed") {
                eprintln!("Failed to create post for site {}: {}", site.id, msg);
            }
        }
    }

    Ok(())
}

async fn block(site: &Model) {
    let count = register_site_error(site.id).await;
    if count >= 5 {
        eprintln!(
            "Site {} reached error threshold ({}), disabling",
            site.id, count
        );
        if let Err(e) = SiteRepository::disable(site.id).await {
            eprintln!("Failed to disable site {}: {}", site.id, e);
        }
    }
}
