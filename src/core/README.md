# PolyChat Core

PolyChat Core is the centralized component that sits between the user interface and the plugin system.

The GUI initializes the core immediately on startup. The GUI then makes calls for things it needs, like getting account, plugin, or conversation info.

## Responsibilities of PolyChat Core

Plugins:
- The core is responsible for interfacing with the plugin system. The GUI should not directly interface with the plugin system.
- The core is responsible for sending and receiving plugin instructions, minus what the plugin system is responsible for.
- TBD: Whether or not the plugin system or core should be responsible for keepalive requests.
- The core is not responsible for isolating itself from a plugin crash. The plugin system is responsible for that. But the core is reasponsible for alerting the GUI so that it may update the GUI to show a crashed plugin and the related error message(s).
- The core is responsible for loading all relevant information from the Init instruction from the plugin.

Accounts:
- The core is responsible for making the login methods for all supported protocols available to the GUI.
- The core is responsible for keeping track of the authenticated and session-expired accounts.

Conversations:
- The core is responsible for keeping track of all conversations for all accounts, and making them available to the GUI.
- The core is responsible for keeping track of all information related to conversations. This includes requesting more messages, and sending messages when the GUI sends the instruction to the core.

Networking:
- The core will not be responsible for handling network connections. Plugins will be more strongly isolated from the core, and will handle it themselves.
- The core will be responsible for aiding in the creation of request IDs that will be associated with any action that is being requested of a plugin.
- The core will be responsible for validating that a plugin does as it is requested. If a request, with an associated request ID, stalls, times out, or does not do what it was supposed to, the core should flag that as a plugin error.
