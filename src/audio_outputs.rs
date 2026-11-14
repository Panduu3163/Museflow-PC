//! Output policy is separate from the native device handles, which stay on the audio worker.
#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    pub id: String,
    pub label: String,
}
pub fn automatic() -> Output {
    Output {
        id: "auto".into(),
        label: "System default".into(),
    }
}
pub fn valid_preference(id: &str) -> bool {
    id == "auto"
        || (id.starts_with("wasapi/")
            && id.len() > 7
            && id.len() <= 512
            && !id.chars().any(char::is_control))
}
pub fn parse(raw: &str) -> Result<Vec<Output>, String> {
    if raw.len() > 65_536 {
        return Err("Audio output list is too large".into());
    }
    let data: serde_json::Value =
        serde_json::from_str(raw).map_err(|_| "Cannot read the audio output list")?;
    let entries = data.as_array().ok_or("Cannot read the audio output list")?;
    let mut result = vec![automatic()];
    for entry in entries.iter().take(128) {
        let Some(id) = entry["name"].as_str() else {
            continue;
        };
        // Windows first: expose concrete WASAPI endpoints, not unrelated driver selectors.
        if id == "auto" || !valid_preference(id) || result.iter().any(|o| o.id == id) {
            continue;
        }
        let label = entry["description"]
            .as_str()
            .unwrap_or("Audio output")
            .chars()
            .filter(|c| !c.is_control())
            .take(160)
            .collect::<String>();
        result.push(Output {
            id: id.into(),
            label: if label.trim().is_empty() {
                "Audio output".into()
            } else {
                label
            },
        });
        if result.len() >= 64 {
            break;
        }
    }
    Ok(result)
}
pub fn effective<'a>(outputs: &'a [Output], preferred: &'a str) -> &'a str {
    if outputs.iter().any(|o| o.id == preferred) {
        preferred
    } else {
        "auto"
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outputs_are_bounded_deduplicated_and_windows_specific() {
        let list=parse(r#"[{"name":"auto"},{"name":"wasapi/speaker","description":"Speakers"},{"name":"wasapi/speaker","description":"Duplicate"},{"name":"openal","description":"Driver"},{"name":"wasapi/bad\u0000","description":"Bad"},{"description":"Missing"}]"#).unwrap();
        assert_eq!(
            list,
            vec![
                automatic(),
                Output {
                    id: "wasapi/speaker".into(),
                    label: "Speakers".into()
                }
            ]
        );
        assert_eq!(effective(&list, "wasapi/missing"), "auto");
        assert_eq!(effective(&list, "wasapi/speaker"), "wasapi/speaker");
        assert!(parse("{}").is_err());
        assert!(parse(&"x".repeat(65_537)).is_err());
        let raw = serde_json::to_string(
            &(0..200)
                .map(|n| serde_json::json!({"name":format!("wasapi/{n}")}))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        assert_eq!(parse(&raw).unwrap().len(), 64);
    }
}
