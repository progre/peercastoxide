use super::console_color::ConsoleColor;

pub fn proxy_message(
    incoming_addr: &str,
    incoming_color: &ConsoleColor,
    local_addr: &str,
    remote_addr: &str,
    remote_color: &ConsoleColor,
) -> String {
    format!(
        "{}{:?}{} --> {:?}(me) --> {}{:?}{}",
        incoming_color.header(),
        incoming_addr,
        incoming_color.footer(),
        local_addr,
        remote_color.header(),
        remote_addr,
        remote_color.footer(),
    )
}
