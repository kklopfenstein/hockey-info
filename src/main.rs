use chrono::{DateTime, Local, Offset, Utc, NaiveDate};
use reqwest::Client;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
struct NhlScoreboardResponse {
    events: Vec<InlineEvent>,
}

#[derive(Debug, Deserialize)]
struct InlineEvent {
    id: Option<String>,
    date: Option<String>,
    time_utc: Option<String>,
    time_zone: Option<String>,
    status: Option<Status>,
    competitors: Option<Vec<Competitor>>,
    venue: Option<Venue>,
    game_number: Option<String>,
     #[allow(dead_code)]
    section: Option<Section>,
    #[allow(dead_code)]
    groups: Option<Groups>,
}

#[derive(Debug, Deserialize)]
struct Status {
     #[allow(dead_code)]
    abstract_code: Option<String>,
    #[allow(dead_code)]
    detail: Option<String>,
    state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Competitor {
    #[allow(dead_code)]
    #[serde(skip)]
    id: Option<String>,
    #[serde(default)]
    team: Option<Team>,
    #[serde(default, alias = "homeAway")]
    home: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Team {
    #[allow(dead_code)]
    #[serde(skip)]
    id: Option<String>,
    #[allow(dead_code)]
    #[serde(skip)]
    name: Option<String>,
    #[allow(dead_code)]
    #[serde(skip)]
    short_name: Option<String>,
    #[allow(dead_code)]
    #[serde(skip)]
    market: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Venue {
    name: Option<String>,
    #[allow(dead_code)]
    #[serde(skip)]
    city: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Section {
    #[allow(dead_code)]
    #[serde(skip)]
    id: Option<String>,
    #[allow(dead_code)]
    #[serde(skip)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Groups {
    #[allow(dead_code)]
    #[serde(skip)]
    id: Option<String>,
    #[allow(dead_code)]
    #[serde(skip)]
    name: Option<String>,
}

#[derive(Debug, Clone)]
struct Game {
    #[allow(dead_code)]
    game_id: String,
    date: NaiveDate,
    date_str: String,
    time: Option<String>,
    time_zone: Option<String>,
    status: Option<String>,
    home_team: String,
    away_team: String,
    #[allow(dead_code)]
    home_market: Option<String>,
    #[allow(dead_code)]
    away_market: Option<String>,
    venue: Option<String>,
    #[allow(dead_code)]
    city: Option<String>,
    #[allow(dead_code)]
    game_number: Option<String>,
    #[allow(dead_code)]
    group_name: Option<String>,
}

impl Game {
    fn from_event(event: &InlineEvent) -> Option<Self> {
        let date_str = event.date.as_ref()?;

        // Handle both formats: YYYY-MM-DDTHH:MM:SSZ (full timestamp) and YYYYMMDD (compact)
        let date_format = if date_str.len() > 10 && date_str.contains('-') {
            // Full timestamp format: 2026-05-30T00:00:00Z, extract just the date part
            date_str[..10].to_string()
        } else if date_str.len() == 8 {
            // Compact format YYYYMMDD, convert to YYYY-MM-DD
            format!("{}-{}-{}", &date_str[..4], &date_str[4..6], &date_str[6..8])
        } else {
            return None;
        };

        // Parse the date string to extract the date
        let dt = DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", date_format))
            .expect("Failed to parse date");
        let date = dt.with_timezone(&Utc).naive_utc().date();
        let date_str_local = date.format("%Y-%m-%d").to_string();

        let home_team = if let Some(ref competitors) = event.competitors {
            let home_competitor = competitors.iter()
                .find(|c| c.home.as_ref().map_or(false, |h| h == "home"))
                .or_else(|| {
                    competitors.iter().find(|c| {
                        c.team.as_ref().map_or(false, |t| {
                            t.id.as_ref().map_or(false, |id| id == "4195")
                        })
                    })
                });
            
            if home_competitor.is_none() {
                return None;
            }
            
            let home_competitor = home_competitor?;
            home_competitor.team.as_ref()
                .and_then(|t| t.name.as_ref())
                .map(|n| n.as_str())
                .unwrap_or("Unknown")
        } else {
            return None;
        };

        let away_team = if let Some(ref competitors) = event.competitors {
            let away_competitor = competitors.iter()
                .find(|c| c.home.as_ref().map_or(false, |h| h == "away"))
                .or_else(|| {
                    competitors.iter().skip(1).find(|c| {
                        c.team.as_ref().map_or(false, |t| {
                            t.id.as_ref().map_or(false, |id| id == "4195")
                        })
                    })
                });

            if away_competitor.is_none() {
                return None;
            }

            let away_competitor = away_competitor?;
            away_competitor.team.as_ref()
                .and_then(|t| t.name.as_ref())
                .map(|n| n.as_str())
                .unwrap_or("Unknown")
        } else {
            return None;
        };

        let game_status = event.status.as_ref().and_then(|s| s.state.as_deref());

        let venue_name = if let Some(ref venue) = event.venue {
            venue.name.as_ref().map(|s| s.as_str().to_string())
        } else {
            None
        };
        let city = if let Some(ref venue) = event.venue {
            venue.city.as_ref().map(|s| s.as_str().to_string())
        } else {
            None
        };

        Some(Game {
            game_id: event.id.clone().unwrap_or_default(),
            date,
            date_str: date_str_local,
            time: event.time_utc.clone(),
            time_zone: event.time_zone.clone(),
            status: game_status.map(|s| s.to_string()),
            home_team: home_team.to_string(),
            away_team: away_team.to_string(),
            home_market: None,
            away_market: None,
            venue: venue_name,
            city,
            game_number: event.game_number.clone(),
            group_name: event.groups.as_ref().and_then(|g| g.name.as_ref().map(|s| s.as_str().to_string())),
        })
    }
}

fn get_local_timezone() -> (DateTime<chrono::Utc>, String) {
    let local = Local::now();
    let utc = local.with_timezone(&chrono::Utc);
    let offset = local.offset().fix().local_minus_utc() as i32;
    let offset_str = if offset > 0 {
        format!("UTC+{}", offset)
    } else {
        format!("UTC{}", offset)
    };
    (utc.clone(), offset_str)
}

fn format_game_time(game: &Game, _utc: &DateTime<chrono::Utc>, offset_str: &str) -> String {
    if let (Some(ref time), Some(ref tz)) = (game.time.clone(), game.time_zone.clone()) {
        format!("{} ({} {})", time, offset_str, tz)
    } else {
        String::new()
    }
}

fn format_game(game: &Game) -> String {
    let mut result = String::new();

    result.push_str(&format!("{}:", game.date_str));
    result.push_str(&format!(" {} @ {}", game.away_team, game.home_team));

    if let (Some(ref game_num), Some(ref group)) = (game.game_number.clone(), game.group_name.clone()) {
        result.push_str(&format!(" (Game {} - {})", game_num, group));
    }

    if let Some(ref venue) = game.venue {
        result.push_str(&format!(" @ {}", venue));
    }

    if let Some(ref city) = game.city {
        result.push_str(&format!(" ({})", city));
    }

    result
}

fn display_schedule(games: &[Game]) {
    let (utc, offset_str) = get_local_timezone();

    if games.is_empty() {
        println!("No games found for the next 7 days.");
        return;
    }

    let mut last_date = None;

    for game in games {
        if Some(&game.date_str) != last_date {
            println!("\n{}:", game.date_str);
            last_date = Some(&game.date_str);
        }

        println!("{}", format_game(game));

        let formatted_time = format_game_time(game, &utc, &offset_str);
        if !formatted_time.is_empty() {
            println!("    {} | {}", game.status.as_deref().unwrap_or("Scheduled"), formatted_time);
        }
    }
}

fn get_games_for_next_7_days(games: Vec<Game>) -> Vec<Game> {
    let now = Local::now();
    let cutoff = now + chrono::Duration::days(7);

    games.into_iter()
        .filter(|game| {
            let game_date = game.date;
            let now_date = now.date_naive();
            let cutoff_date = cutoff.date_naive();
            let in_range = game_date >= now_date && game_date <= cutoff_date;
            in_range
        })
        .collect()
}

async fn fetch_nhl_schedule() -> Result<Vec<Game>, String> {
    let client = Client::new();
    let url = "https://site.api.espn.com/apis/site/v2/sports/hockey/nhl/scoreboard";

    let response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch data: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()));
    }

    let body = response.text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

 

    let nhl_response: NhlScoreboardResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut games: Vec<Game> = Vec::new();

    for event in nhl_response.events {
        if let Some(game) = Game::from_event(&event) {
            games.push(game);
        }
    }

    Ok(games)
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let games = runtime.block_on(async {
        match fetch_nhl_schedule().await {
            Ok(g) => g,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    });

    let filtered_games = get_games_for_next_7_days(games);
    display_schedule(&filtered_games);
}

#[cfg(test)]
mod tests {
    use super::*;
 

    #[test]
    fn test_date_parsing() {
        let _date_str = "20250115";
        let parsed = DateTime::parse_from_rfc3339(&format!("2025-01-15T00:00:00Z"))
            .unwrap()
            .with_timezone(&Local);
        assert!(parsed.format("%Y-%m-%d").to_string().starts_with("2025"));
    }

    #[test]
    fn test_game_from_event() {
        let mock_event = InlineEvent {
            id: Some("12345".to_string()),
            date: Some("20250115".to_string()),
            time_utc: Some("2025-01-15T19:00:00Z".to_string()),
            time_zone: Some("America/New_York".to_string()),
            status: Some(Status {
                abstract_code: Some("1".to_string()),
                state: Some("pre".to_string()),
                detail: Some("Scheduled".to_string()),
            }),
            competitors: Some(vec![
                Competitor {
                    id: None,
                    team: Some(Team {
                        id: Some("4194".to_string()),
                        name: Some("Toronto Maple Leafs".to_string()),
                        short_name: Some("TOR".to_string()),
                        market: Some("Toronto".to_string()),
                    }),
                    home: Some("home".to_string()),
                },
                Competitor {
                    id: None,
                    team: Some(Team {
                        id: Some("4195".to_string()),
                        name: Some("Montreal Canadiens".to_string()),
                        short_name: Some("MTL".to_string()),
                        market: Some("Montreal".to_string()),
                    }),
                    home: Some("away".to_string()),
                },
            ]),
            venue: Some(Venue {
                name: Some("Bell Centre".to_string()),
                city: Some("Montreal".to_string()),
            }),
            game_number: Some("1".to_string()),
            section: None,
            groups: Some(Groups {
                id: Some("1".to_string()),
                name: Some("Atlantic Division".to_string()),
            }),
        };

        let game = Game::from_event(&mock_event).expect("Failed to create game from event");
        assert_eq!(game.away_team, "Montreal Canadiens");
        assert_eq!(game.home_team, "Toronto Maple Leafs");
        assert_eq!(game.date_str, "2025-01-15");
        assert_eq!(game.venue, Some("Bell Centre".to_string()));
        assert_eq!(game.city, Some("Montreal".to_string()));
        assert_eq!(game.group_name, Some("Atlantic Division".to_string()));
    }

    #[test]
    fn test_filter_games() {
        let now = Local::now();
        let seven_days_ago = now - chrono::Duration::days(8);
        let five_days_from_now = now + chrono::Duration::days(5);
        let future_date = now + chrono::Duration::days(10);

        let mock_games = vec![
            Game {
                game_id: "1".to_string(),
                date: seven_days_ago.date_naive(),
                date_str: seven_days_ago.format("%Y-%m-%d").to_string(),
                time: None,
                time_zone: None,
                status: None,
                home_team: "Team A".to_string(),
                away_team: "Team B".to_string(),
                home_market: None,
                away_market: None,
                venue: None,
                city: None,
                game_number: None,
                group_name: None,
            },
            Game {
                game_id: "2".to_string(),
                date: five_days_from_now.date_naive(),
                date_str: five_days_from_now.format("%Y-%m-%d").to_string(),
                time: None,
                time_zone: None,
                status: None,
                home_team: "Team C".to_string(),
                away_team: "Team D".to_string(),
                home_market: None,
                away_market: None,
                venue: None,
                city: None,
                game_number: None,
                group_name: None,
            },
            Game {
                game_id: "3".to_string(),
                date: future_date.date_naive(),
                date_str: future_date.format("%Y-%m-%d").to_string(),
                time: None,
                time_zone: None,
                status: None,
                home_team: "Team E".to_string(),
                away_team: "Team F".to_string(),
                home_market: None,
                away_market: None,
                venue: None,
                city: None,
                game_number: None,
                group_name: None,
            },
        ];

        let filtered = get_games_for_next_7_days(mock_games);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].game_id, "2");
    }

    #[test]
    fn test_format_game() {
        let game = Game {
            game_id: "1".to_string(),
            date: Local::now().date_naive(),
            date_str: "2025-01-15".to_string(),
            time: Some("19:00".to_string()),
            time_zone: Some("America/New_York".to_string()),
            status: Some("Scheduled".to_string()),
            home_team: "Home Team".to_string(),
            away_team: "Away Team".to_string(),
            home_market: None,
            away_market: None,
            venue: Some("Madison Square Garden".to_string()),
            city: Some("New York".to_string()),
            game_number: Some("1".to_string()),
            group_name: Some("Eastern Conference".to_string()),
        };

        let formatted = format_game(&game);
        assert!(formatted.contains("2025-01-15"));
        assert!(formatted.contains("Away Team @ Home Team"));
    }

    #[test]
    fn test_timezone_offset() {
        let (_, offset_str) = get_local_timezone();
        assert!(!offset_str.is_empty());
    }
}
