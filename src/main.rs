use std::error::Error;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

struct App {
    pub id: String,
    pub name: String,
}

impl App {
    fn new(id: &str, name: &str) -> Self {
        App {
            id: id.to_string(),
            name: name.to_string(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut apps: Vec<App> = Vec::new();
    
    // replace here with your app id and name
    apps.push(App::new("1462608349", "Octofile"));

    for app in apps {
        println!("try get app store reviews for app: {}", app.name.clone());

        // get reviews
        let scraper = AppStoreScraper::new(app.id.clone());
        let reviews = scraper.get_reviews().await?;

        // export to csv, or do you want to do something else?
        let mut writer = csv::Writer::from_path(format!("{0}-reviews.csv", app.name.clone()))?;
        writer.write_record(vec![
            "ID", "Score", "Name", "Title", "Text", "Updated", "Channel",
        ])?;
        for review in reviews {
            writer.write_record(review)?;
        }
        writer.flush()?;
    }

    Ok(())
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Review {
    pub id: String,
    pub score: String,
    pub name: String,
    pub title: String,
    pub text: String,
    pub updated: String,
    pub channel: String,
}

impl IntoIterator for Review {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            self.id,
            self.score,
            self.name,
            self.title,
            self.text,
            self.updated,
            self.channel,
        ]
        .into_iter()
    }
}

impl Review {
    fn new(
        id: String,
        score: String,
        name: String,
        title: String,
        text: String,
        updated: String,
        channel: String,
    ) -> Self {
        Review {
            id,
            score,
            name,
            title,
            text,
            updated,
            channel,
        }
    }
}

type Reviews = Vec<Review>;

#[async_trait]
trait Scraper {
    async fn get_reviews(&self) -> Result<Reviews, Box<dyn Error>>;
}
struct AppStoreScraper {
    app_id: String,
    regions: Vec<String>,
}

impl AppStoreScraper {
    fn new(app_id: String) -> Self {
        Self {
            app_id,
            regions: vec!["cn".to_string(), "us".to_string()],
        }
    }

    async fn get_reviews_with_region(
        &self,
        region: Option<String>,
    ) -> Result<Reviews, Box<dyn Error>> {
        let mut reviews: Reviews = Vec::new();
        for index in 1..=10 {
            let url: String = if let Some(value) = region.clone() {
                format!(
                    "https://itunes.apple.com/{0}/rss/customerreviews/id={1}/sortBy=mostRecent/page={2}/json", value, self.app_id, index)
            } else {
                format!(
                    "https://itunes.apple.com/rss/customerreviews/id={0}/sortBy=mostRecent/page={1}/json", self.app_id, index)
            };

            let resp = reqwest::get(url).await?;
            let body = resp.text().await.unwrap_or_default();
            let values: AppStoreReviews = serde_json::from_str(&body).unwrap_or_default();
            if values.feed.entry.is_empty() {
                println!("no more reviews for app: {}, source: {}", self.app_id, body);
            }
            let items: Reviews = values
                .feed
                .entry
                .into_iter()
                .map(|value| Review::from(&value))
                .collect();
            reviews.extend(items);
        }
        Ok(reviews)
    }
}

#[derive(Debug, Deserialize, Default)]
struct AppStoreReviewEntry {
    pub label: String,
}

#[derive(Debug, Deserialize, Default)]
struct AppStoreReviewAuthor {
    pub name: AppStoreReviewEntry,
}

#[derive(Debug, Deserialize, Default)]
struct AppStoreReview {
    pub id: AppStoreReviewEntry,
    #[serde(alias = "im:rating", default)]
    pub score: AppStoreReviewEntry,
    pub author: AppStoreReviewAuthor,
    pub title: AppStoreReviewEntry,
    pub content: AppStoreReviewEntry,
    pub updated: AppStoreReviewEntry,
}

impl From<&AppStoreReview> for Review {
    fn from(review: &AppStoreReview) -> Self {
        Review::new(
            review.id.label.clone(),
            review.score.label.clone(),
            review.author.name.label.clone(),
            review.title.label.clone(),
            review.content.label.clone(),
            review.updated.label.clone(),
            "App Store".to_string(),
        )
    }
}

#[derive(Debug, Deserialize, Default)]
struct AppStoreReviews {
    pub feed: AppStoreReviewFeed,
}

#[derive(Debug, Deserialize, Default)]
struct AppStoreReviewFeed {
    #[serde(default)]
    pub entry: Vec<AppStoreReview>,
}

#[async_trait]
impl Scraper for AppStoreScraper {
    async fn get_reviews(&self) -> Result<Reviews, Box<dyn Error>> {
        let mut reviews: Reviews = Vec::new();
        // get default reviews
        let region_reviews = self.get_reviews_with_region(None).await?;
        reviews.extend(region_reviews);
        // get reviews by region
        for region in &self.regions {
            let region_reviews = self.get_reviews_with_region(Some(region.clone())).await?;
            reviews.extend(region_reviews);
        }
        Ok(reviews)
    }
}
