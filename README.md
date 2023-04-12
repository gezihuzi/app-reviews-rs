# App Store Reviews

This is a collection of reviews from the App Store.

In main function, you can put any app info and get the reviews.

Looks like:

```rust
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
```

Currently only supports App Store.
