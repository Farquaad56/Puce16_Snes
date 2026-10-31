//! Cheat code collection from file and CLI arguments (E10-04).

use puce16_core::cheat::{parse_cheat_file, parse_par, ParCode};

/// Collect PAR codes from an optional cheat file and/or individual --cheat-code args.
/// File entries come first (only enabled ones), then each CLI code in order.
pub fn collect_cheats(file_text: Option<&str>, codes: &[String]) -> Result<Vec<ParCode>, String> {
    let mut result = Vec::new();

    if let Some(text) = file_text {
        match parse_cheat_file(text) {
            Ok(entries) => {
                for entry in entries {
                    if entry.enabled {
                        result.extend(entry.codes);
                    }
                }
            }
            Err(e) => {
                return Err(format!("fichier .cht ligne {}: {:?}", e.line, e.kind));
            }
        }
    }

    for code_str in codes {
        match parse_par(code_str) {
            Ok(code) => result.push(code),
            Err(e) => {
                return Err(format!("code invalide '{}': {:?}", code_str, e));
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e10_04_collect_only_enabled_in_order() {
        let file = "+ A = 7E0001AA\n- B = 7E0002BB\n+ C = 7E0003CC 7E0004DD";
        let codes = vec!["7E0005EE".to_string()];
        let result = collect_cheats(Some(file), &codes).unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0].addr, 0x7E0001);
        assert_eq!(result[0].value, 0xAA);
        assert_eq!(result[1].addr, 0x7E0003);
        assert_eq!(result[1].value, 0xCC);
        assert_eq!(result[2].addr, 0x7E0004);
        assert_eq!(result[2].value, 0xDD);
        assert_eq!(result[3].addr, 0x7E0005);
        assert_eq!(result[3].value, 0xEE);
    }

    #[test]
    fn e10_04_collect_without_file() {
        let codes = vec!["7E0010FF".to_string()];
        let result = collect_cheats(None, &codes).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].addr, 0x7E0010);
        assert_eq!(result[0].value, 0xFF);

        let empty = collect_cheats(None, &[]).unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn e10_04_collect_bad_code_message() {
        let codes = vec!["7E0010F".to_string()];
        let err = collect_cheats(None, &codes).unwrap_err();
        assert!(err.contains("code invalide '7E0010F'"));
        assert!(err.contains("Length(7)"));
    }

    #[test]
    fn e10_04_collect_bad_file_message() {
        let err = collect_cheats(Some("oops"), &[]).unwrap_err();
        assert!(err.contains("ligne 1"));
        assert!(err.contains("MissingSign"));
    }
}
