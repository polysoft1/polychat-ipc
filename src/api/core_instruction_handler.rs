use super::schema::api::*;

/// A trait to be implemented by the core for instructions sent from a plugin
/// Also can be implemented by the plugin SDK, which can then be translated
/// to instructions, and back again in the core.
pub trait CoreInstructionHandler {
    fn on_init(data: InitDataInstruction);
    fn on_keepalive_response(response: KeepaliveInstruction);
    fn on_auth_account_response(response: AuthAccountResponse);
}
