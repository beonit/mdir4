use std::collections::HashSet;

use crate::layout::text::cell_width;

use super::api::{FileDecoration, StatusItem, StyledText};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContributionFault {
    DuplicateId(String),
    DecorationExceedsReservation { entry_id: String },
}

pub fn validate_decorations(
    decorations: Vec<FileDecoration>,
) -> Result<Vec<FileDecoration>, ContributionFault> {
    let mut entries = HashSet::new();
    for decoration in &decorations {
        if !entries.insert(decoration.entry_id.clone()) {
            return Err(ContributionFault::DuplicateId(decoration.entry_id.clone()));
        }
        let width = decoration
            .text
            .spans
            .iter()
            .map(|span| cell_width(&span.text))
            .sum::<usize>();
        if width > usize::from(decoration.reserved_cells) {
            return Err(ContributionFault::DecorationExceedsReservation {
                entry_id: decoration.entry_id.clone(),
            });
        }
    }
    Ok(decorations)
}

pub fn status_for_width(items: &[StatusItem], width: u16) -> Vec<StyledText> {
    let mut ordered = items.to_vec();
    ordered.sort_by_key(|item| item.priority);
    let mut used = 0usize;
    let mut output = Vec::new();
    for item in ordered {
        let full = text_width(&item.full);
        let compact = text_width(&item.compact);
        let available = usize::from(width).saturating_sub(used);
        let selected = if full <= available {
            Some(item.full)
        } else if compact <= available {
            Some(item.compact)
        } else {
            None
        };
        if let Some(text) = selected {
            used += text_width(&text);
            output.push(text);
        }
    }
    output
}

fn text_width(text: &StyledText) -> usize {
    text.spans.iter().map(|span| cell_width(&span.text)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::api::{StatusItem, StyledSpan};

    fn text(value: &str) -> StyledText {
        StyledText {
            spans: vec![StyledSpan {
                text: value.into(),
                role: None,
            }],
        }
    }

    #[test]
    fn decorations_respect_reserved_cells_and_status_degrades_full_compact_hidden() {
        assert!(matches!(
            validate_decorations(vec![FileDecoration {
                entry_id: "a".into(),
                text: text("abc"),
                reserved_cells: 2,
                priority: 1
            }]),
            Err(ContributionFault::DecorationExceedsReservation { .. })
        ));
        let items = vec![
            StatusItem {
                id: "one".into(),
                full: text("Branch main "),
                compact: text("main "),
                priority: 1,
            },
            StatusItem {
                id: "two".into(),
                full: text("3 changes"),
                compact: text("3"),
                priority: 2,
            },
        ];
        assert_eq!(status_for_width(&items, 21).len(), 2);
        assert_eq!(status_for_width(&items, 7)[0].spans[0].text, "main ");
        assert!(status_for_width(&items, 0).is_empty());
    }
}
