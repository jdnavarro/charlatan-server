use std::collections::HashMap;
use std::convert::Infallible;

use diesel::prelude::{QueryDsl, RunQueryDsl};
use rss::Channel;
use warp::http::StatusCode;

use super::models::{NewEpisode, NewPodcast, Podcast, PooledSqliteConnection};
use super::schema;

pub async fn list_podcasts(conn: PooledSqliteConnection) -> Result<impl warp::Reply, Infallible> {
    let results = schema::podcast::table
        .load::<Podcast>(&conn)
        .expect("Error loading posts");
    Ok(warp::reply::json(&results))
}

pub async fn get_podcasts(
    id: i32,
    conn: PooledSqliteConnection,
) -> Result<impl warp::Reply, Infallible> {
    let results = schema::podcast::table
        .find(id)
        .load::<Podcast>(&conn)
        .expect("Error loading posts");
    Ok(warp::reply::json(&results))
}

pub async fn add_podcast(
    hm: HashMap<String, String>,
    conn: PooledSqliteConnection,
) -> Result<impl warp::Reply, Infallible> {
    let title = hm.get("title").unwrap();
    let url = hm.get("url").unwrap();

    let new_podcast = NewPodcast { title, url };

    let results = diesel::insert_into(schema::podcast::table)
        .values(&new_podcast)
        .execute(&conn)
        .expect("Error saving new podcast");

    Ok(warp::reply::json(&results))
}

pub async fn fetch_episodes(conn: PooledSqliteConnection) -> Result<impl warp::Reply, Infallible> {
    let podcasts = schema::podcast::table
        .load::<Podcast>(&conn)
        .expect("Error loading posts");

    for podcast in podcasts {
        let channel = Channel::from_url(&podcast.url).unwrap();

        for episode_item in channel.items() {
            let new_episode = NewEpisode {
                title: &episode_item.title().unwrap(),
                url: &episode_item.enclosure().unwrap().url(),
                podcast_id: &podcast.id,
            };
            diesel::insert_into(schema::episode::table)
                .values(&new_episode)
                .execute(&conn)
                .expect("Error saving new episode");
        }
    }
    Ok(StatusCode::CREATED)
}