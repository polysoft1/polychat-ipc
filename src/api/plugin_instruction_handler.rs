use super::schema::{
    auth::AuthAccountInstruction,
    keepalive::KeepaliveInstruction,
};
/// A trait to be implemented by the core for instructions sent from a plugin
/// Also can be implemented by the plugin SDK, which can then be translated
/// to instructions, and back again in the core.
pub trait PluginInstructionHandler {
    fn on_keepalive(data: KeepaliveInstruction);
    fn on_auth_account(data: AuthAccountInstruction);
}