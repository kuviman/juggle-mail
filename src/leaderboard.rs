use super::*;

pub async fn submit(diff: Difficulty, name: &str, score: f32) -> (usize, Vec<jornet::Score>) {
    let mut leaderboard = jornet::Leaderboard::with_host_and_leaderboard(
        None,
        "d5e902bf-cf5d-4bb4-9472-729cd5e2d5aa".parse().unwrap(),
        "e534331f-f0fc-4d5e-89f2-d19928b7f633".parse().unwrap(),
    );
    let player = if let Some(player) = preferences::load::<jornet::Player>("player") {
        log::info!("Returning player");
        if player.name == name {
            leaderboard.as_player(player.clone());
            player
        } else {
            log::info!("Name has changed");
            let player = leaderboard.create_player(Some(name)).await.unwrap();
            preferences::save("player", player);
            player.clone()
        }
    } else {
        log::info!("New player");
        let player = leaderboard.create_player(Some(name)).await.unwrap();
        preferences::save("player", &player);
        player.clone()
    };
    let meta = serde_json::to_string(&diff).unwrap();
    leaderboard
        .send_score_with_meta(score, &meta)
        .await
        .unwrap();
    let mut scores = leaderboard.get_leaderboard().await.unwrap();
    scores.retain(|score| score.meta.as_deref() == Some(meta.as_str()));
    scores.sort_by_key(|score| -r32(score.score));

    {
        // Only leave unique names
        let mut i = 0;
        let mut names_seen = HashSet::new();
        while i < scores.len() {
            if !names_seen.contains(&scores[i].player) {
                names_seen.insert(scores[i].player.clone());
                i += 1;
            } else if scores[i].score == score {
                i += 1;
            } else {
                scores.remove(i);
            }
        }
    }

    let my_pos = scores.iter().position(|this| this.score == score).unwrap();

    {
        // Only leave unique names
        let mut i = 0;
        let mut names_seen = HashSet::new();
        while i < scores.len() {
            if !names_seen.contains(&scores[i].player) {
                names_seen.insert(scores[i].player.clone());
                i += 1;
            } else {
                scores.remove(i);
            }
        }
    }
    scores.truncate(5);
    (my_pos, scores)
}
