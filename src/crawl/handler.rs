use chrono::DateTime;
use warp::http::StatusCode;

use crate::{app::App, crawl, episode::NewEpisode, podcast::Podcast};

pub(super) async fn crawl(
    _identity: String,
    app: App,
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    match app.podcast.list().await {
        Ok(podcasts) => {
            // TODO: Stream directly with sqlx cursor?
            for podcast in podcasts {
                match crawl_podcast(app.clone(), &podcast).await {
                    Ok(()) => log::info!("Podcast crawled: {}", &podcast.id),
                    Err(e) => {
                        log::warn!("Skipping podcast {} because err -- {:#?}", &podcast.id, &e)
                    }
                };
            }
            Ok(StatusCode::CREATED)
        }

        Err(e) => {
            log::error!("Error while listing podcasts -- {:#?}", &e);
            Ok(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

type Result<T> = std::result::Result<T, crawl::Error>;

async fn crawl_podcast(app: App, podcast: &Podcast) -> Result<()> {
    let channel = rss::Channel::from_url(&podcast.src)?;

    for item in channel.items() {
        let new_episode = parse(&podcast, &item)?;
        app.episode.add(&new_episode).await?;
    }
    Ok(())
}

fn parse<'a>(podcast: &'a Podcast, item: &'a rss::Item) -> Result<NewEpisode<'a>> {
    let src = &item
        .enclosure()
        .ok_or(crawl::Error::MissingSource(podcast.id))?
        .url();

    let guid = &item.guid().map_or_else(
        || {
            log::warn!(
                "Missing guid in episode for podcast id: {}, using source instead: {}",
                podcast.id,
                &src
            );
            *src
        },
        |i| i.value(),
    );

    let title = &item.title().unwrap_or_else(|| {
        log::warn!(
            "Missing title in episode guid: {}. Using source URL: {}",
            &guid,
            &src
        );
        *src
    });

    let duration = item
        .itunes_ext()
        .and_then(|it| it.duration())
        .and_then(parse_duration)
        .unwrap_or_else(|| {
            log::warn!("Missing duration for episode guid: {}", &guid);
            0
        });

    let image = item
        .itunes_ext()
        .and_then(|it| it.image())
        .unwrap_or_else(|| {
            log::info!(
                "Missing image for episode guid: {}. Using podcast image",
                &guid
            );
            &podcast.image
        });

    let publication = item.pub_date().map_or_else(
        || {
            log::warn!("Missing publication date for episode guid: {}", &guid);
            0
        },
        |d| {
            DateTime::parse_from_rfc2822(&d).map_or_else(
                |e| {
                    log::warn!(
                        "Failed to parse publication date for episode. | guid: {} | publication date: {} -- {:#?}",
                        &guid,
                        &d,
                        &e
                    );
                    0
                },
                |x| x.timestamp(),
            )
        },
    );

    let notes = item.description().unwrap_or_else(|| {
        log::warn!("Missing description for episode guid: {}", &guid);
        ""
    });

    Ok(NewEpisode {
        title,
        guid,
        duration,
        image,
        publication,
        src,
        notes,
        podcast: podcast.id,
    })
}

fn parse_duration(s: &str) -> Option<i64> {
    let mut x = s.split(|c| c == ':').map(|e| e.parse::<i64>());

    let secs = match x.next() {
        None => {
            log::warn!("Duration empty");
            None
        }
        Some(Err(e)) => {
            log::warn!("Couldn't parse seconds in duration: {} -- {:#?}", &s, &e);
            None
        }
        Some(Ok(r)) => Some(r),
    }?;

    let mins = match x.next() {
        None => Some(0),
        Some(Err(e)) => {
            log::warn!("Couldn't parse minutes in duration: {} -- {:#?}", &s, &e);
            None
        }
        Some(Ok(r)) => Some(r * 60),
    }?;

    let hours = match x.next() {
        None => Some(0),
        Some(Err(e)) => {
            log::warn!("Couldn't parse hours in duration: {} -- {:#?}", &s, &e);
            None
        }
        Some(Ok(r)) => Some(r * 3600),
    }?;

    match x.next() {
        None => Some(secs + mins + hours),
        Some(_) => {
            log::warn!("Couldn't understand duration format: {}", &s);
            None
        }
    }
}
