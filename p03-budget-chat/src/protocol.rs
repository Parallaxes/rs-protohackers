pub fn is_valid_name(name: &str) -> bool {
    // Check length and alphanumeric
    !name.is_empty() && name.len() <= 16 && name.chars().all(|c| c.is_ascii_alphanumeric())
}

pub fn format_join_message(name: &str) -> String {
    format!("* {} has entered the room\n", name)
}

pub fn format_leave_message(name: &str) -> String {
    format!("* {} has left the room\n", name)
}

pub fn format_user_list(users: &[String]) -> String {
    let list = users.join(", ");
    format!("* The room contains: {}\n", list)
}