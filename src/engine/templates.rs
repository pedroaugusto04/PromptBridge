use std::fs;

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
        let base_instructions = Self::load_template("system_prompt_base.txt")
            .replace("{target_language}", target_language);

        let template_name = match mode {
            TransformMode::Translate => "translate.txt",
            TransformMode::Optimize => "optimize.txt",
            TransformMode::Transform => "transform.txt",
        };

        let template = Self::load_template(template_name);
        template
            .replace("{base_instructions}", &base_instructions)
            .replace("{target_language}", target_language)
    }

    /// Load a template file from the templates directory
    fn load_template(filename: &str) -> String {
        // Try to load from the templates directory relative to the executable
        let template_path = format!("templates/{}", filename);
        
        // If running from the project directory, try relative path
        if let Ok(content) = fs::read_to_string(&template_path) {
            return content;
        }

        // Fallback: try to find templates in the same directory as the executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let fallback_path = exe_dir.join("templates").join(filename);
                if let Ok(content) = fs::read_to_string(&fallback_path) {
                    return content;
                }
            }
        }

        // Final fallback to hardcoded templates
        Self::get_fallback_template(filename)
    }

    /// Fallback hardcoded templates for when file loading fails
    fn get_fallback_template(filename: &str) -> String {
        match filename {
            "system_prompt_base.txt" => {
                "You are PromptBridge, an expert AI prompt engineering system.\n\
                 STRICT RULES:\n\
                 1. Preserved Tokens: All placeholders formatted like `__PB_CODE_BLOCK_X__`, `__PB_CODE_INLINE_X__`, `__PB_PATH_X__` MUST BE PRESERVED EXACTLY AS WRITTEN. Do not alter, translate, remove, or modify them.\n\
                 2. Technical Terms: Do NOT translate programming languages, framework names (e.g., Tokio, React, Serde, Cargo), library functions, API routes, or CLI flags.\n\
                 3. Output Format: Return ONLY the transformed prompt text. Do NOT include markdown code blocks around the entire output, meta commentary, or greetings.\n\
                 4. Target Language: The natural language portion of the prompt must be in '{target_language}'."
                    .to_string()
            }
            "translate.txt" => {
                "{base_instructions}\n\n\
                 Translate to: {target_language}\n\n\
                 Rules:\n\
                 - Preserve meaning and sentence structure.\n\
                 - Do not paraphrase.\n\
                 - Keep code, placeholders, paths, commands, and identifiers unchanged.\n\
                 - Return only the translation."
                    .to_string()
            }
            "optimize.txt" => {
                "{base_instructions}\n\n\
                 TASK: Expand and optimize the developer's prompt for an AI Coding Agent (e.g., Claude Code, Cursor, Aider).\n\
                 Structure the prompt clearly with:\n\
                 - Objective / Goal\n\
                 - Context & Technical Constraints\n\
                 - Step-by-Step Implementation Instructions\n\
                 - Verification / Testing expectations."
                    .to_string()
            }
            "transform.txt" => {
                "{base_instructions}\n\n\
                 TASK: Both TRANSLATE natural language to '{target_language}' AND OPTIMIZE the instruction structure for AI Coding Agents.\n\
                 Make the prompt explicit, highly actionable, structured, and easy for an automated developer agent to execute directly."
                    .to_string()
            }
            _ => String::new(),
        }
    }
}
