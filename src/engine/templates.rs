use crate::constants::SYSTEM_PROMPT_BASE_INSTRUCTIONS;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformMode {
    Translate,
    Optimize,
    Transform,
}

pub struct TemplateEngine;

impl TemplateEngine {
    /// Returns the appropriate system prompt for the desired transformation mode and target language
    pub fn get_system_prompt(mode: TransformMode, target_language: &str) -> String {
        let base_instructions = SYSTEM_PROMPT_BASE_INSTRUCTIONS
            .replace("{target_language}", target_language);

        match mode {
            TransformMode::Translate => format!(
                "{}\n\n\
                 TASK: Translate the natural language text into clean, professional '{}'. Keep all code, placeholders, and technical identifiers identical.",
                base_instructions, target_language
            ),
            TransformMode::Optimize => format!(
                "{}\n\n\
                 TASK: Expand and optimize the developer's prompt for an AI Coding Agent (e.g., Claude Code, Cursor, Aider).\n\
                 Structure the prompt clearly with:\n\
                 - Objective / Goal\n\
                 - Context & Technical Constraints\n\
                 - Step-by-Step Implementation Instructions\n\
                 - Verification / Testing expectations.",
                base_instructions
            ),
            TransformMode::Transform => format!(
                "{}\n\n\
                 TASK: Both TRANSLATE natural language to '{}' AND OPTIMIZE the instruction structure for AI Coding Agents.\n\
                 Make the prompt explicit, highly actionable, structured, and easy for an automated developer agent to execute directly.",
                base_instructions, target_language
            ),
        }
    }
}
