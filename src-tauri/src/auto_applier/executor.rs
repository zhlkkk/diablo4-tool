use crate::auto_applier::coords::{ParagonBoardCoords, SkillTreeCoords};
use crate::types::{Resolution, Variant};

/// A single mouse click step in the automation sequence.
#[derive(Debug, Clone)]
pub struct ClickStep {
    pub x: u32,        // 1080p reference x
    pub y: u32,        // 1080p reference y
    pub label: String, // Human-readable description for progress events
}

/// Build an ordered sequence of ClickSteps from a Variant at 1080p reference coordinates.
/// Order: skills (per skill_order or skill.keys()), then equip_skills, then paragon boards.
///
/// Coordinates are 1080p reference values; scale_coord() is applied later in run().
pub fn build_step_sequence(variant: &Variant, _res: &Resolution) -> Vec<ClickStep> {
    let mut steps: Vec<ClickStep> = Vec::new();

    // Phase 1: Skills — iterate in skill_order, or fall back to sorted skill.keys()
    let order: Vec<u32> = if !variant.skill_order.is_empty() {
        variant.skill_order.clone()
    } else {
        let mut keys: Vec<u32> = variant.skill.keys().cloned().collect();
        keys.sort();
        keys
    };

    for (idx, &skill_id) in order.iter().enumerate() {
        let rank = match variant.skill.get(&skill_id) {
            Some(&r) if r > 0 => r,
            _ => continue,
        };
        for n in 1..=rank {
            let x = SkillTreeCoords::ALLOCATE_BUTTON.x
                + (idx as u32) * SkillTreeCoords::SKILL_GRID_SPACING;
            let y = SkillTreeCoords::ALLOCATE_BUTTON.y;
            steps.push(ClickStep {
                x,
                y,
                label: format!("Skill {} point {}/{}", skill_id, n, rank),
            });
        }
    }

    // Phase 2: Equip skills
    for equip in &variant.equip_skills {
        steps.push(ClickStep {
            x: SkillTreeCoords::ALLOCATE_BUTTON.x,
            y: SkillTreeCoords::ALLOCATE_BUTTON.y,
            label: format!("Equip skill {}", equip.key),
        });
    }

    // Phase 3: Paragon boards — sorted by index
    let mut boards = variant.paragon.clone();
    boards.sort_by_key(|b| b.index);

    for board in &boards {
        // Navigate to this board (nav click for boards after the first)
        if board.index > 0 {
            steps.push(ClickStep {
                x: ParagonBoardCoords::BOARD_NAV_NEXT.x,
                y: ParagonBoardCoords::BOARD_NAV_NEXT.y,
                label: format!("Navigate to paragon board {}", board.name),
            });
        }
        // One click per node
        for node_id in &board.nodes {
            steps.push(ClickStep {
                x: ParagonBoardCoords::CENTER.x,
                y: ParagonBoardCoords::CENTER.y,
                label: format!("Paragon {} node {}", board.name, node_id),
            });
        }
    }

    steps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EquipSkill, ParagonBoard, Resolution, Variant};
    use std::collections::HashMap;

    fn empty_variant() -> Variant {
        Variant {
            name: "test".to_string(),
            skill: HashMap::new(),
            skill_order: vec![],
            equip_skills: vec![],
            paragon: vec![],
        }
    }

    #[test]
    fn test_step_sequence_skills_before_paragon() {
        // 2 skills (rank 1 each) + 1 paragon board (1 node)
        let mut variant = empty_variant();
        variant.skill.insert(1, 1);
        variant.skill.insert(2, 1);
        variant.paragon.push(ParagonBoard {
            name: "Board1".to_string(),
            index: 0,
            rotate: 0,
            nodes: vec!["node_a".to_string()],
            glyph: None,
        });

        let steps = build_step_sequence(&variant, &Resolution::Res1080p);

        // Should have 2 skill steps + 1 paragon step = 3 total
        assert_eq!(steps.len(), 3, "Expected 3 steps, got {}", steps.len());

        // Verify skill steps come before paragon steps
        let skill_indices: Vec<usize> = steps
            .iter()
            .enumerate()
            .filter(|(_, s)| s.label.starts_with("Skill "))
            .map(|(i, _)| i)
            .collect();
        let paragon_indices: Vec<usize> = steps
            .iter()
            .enumerate()
            .filter(|(_, s)| s.label.starts_with("Paragon "))
            .map(|(i, _)| i)
            .collect();

        assert!(!skill_indices.is_empty(), "No skill steps found");
        assert!(!paragon_indices.is_empty(), "No paragon steps found");
        assert!(
            skill_indices.iter().all(|&si| paragon_indices.iter().all(|&pi| si < pi)),
            "Skill steps must all come before paragon steps"
        );
    }

    #[test]
    fn test_step_sequence_respects_skill_order() {
        // skill_order=[2, 1] means skill 2 should produce steps before skill 1
        let mut variant = empty_variant();
        variant.skill.insert(1, 1);
        variant.skill.insert(2, 1);
        variant.skill_order = vec![2, 1];

        let steps = build_step_sequence(&variant, &Resolution::Res1080p);

        assert_eq!(steps.len(), 2, "Expected 2 steps");
        assert!(
            steps[0].label.contains("Skill 2"),
            "First step label should reference Skill 2, got: {}",
            steps[0].label
        );
        assert!(
            steps[1].label.contains("Skill 1"),
            "Second step label should reference Skill 1, got: {}",
            steps[1].label
        );
    }

    #[test]
    fn test_step_sequence_empty_variant() {
        let variant = empty_variant();
        let steps = build_step_sequence(&variant, &Resolution::Res1080p);
        assert!(steps.is_empty(), "Empty variant should produce no steps");
    }

    #[test]
    fn test_step_sequence_equip_after_skill() {
        // 1 skill + 1 equip_skill — equip step must come after skill step
        let mut variant = empty_variant();
        variant.skill.insert(5, 1);
        variant.equip_skills.push(EquipSkill {
            key: "slot_1".to_string(),
            mods: vec![],
            rank: 0,
        });

        let steps = build_step_sequence(&variant, &Resolution::Res1080p);

        assert_eq!(steps.len(), 2, "Expected 2 steps");
        assert!(
            steps[0].label.starts_with("Skill "),
            "First step should be a skill step, got: {}",
            steps[0].label
        );
        assert!(
            steps[1].label.starts_with("Equip skill "),
            "Second step should be an equip step, got: {}",
            steps[1].label
        );
    }
}
