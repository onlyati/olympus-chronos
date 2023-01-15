use crate::enums::command_output_type::CommandOutputType;

pub struct CommandOutput {
    pub time: String,
    pub text: String,
    pub r#type: CommandOutputType,
}