# PolyChat Plugin SDK Rust

The PolyChat Plugin SDK Rust is an official plugin SDK written in Rust allow your plugin to interface with PolyChat.

Components:
- SocketCommunicator: Handles IPC communication.
- entrypoint::run_plugin: A function that handles the expected behavior when the plugin is launched. This includes getting the socket/pipe ID, starting the SocketCommunicator, and sending the init instruction.