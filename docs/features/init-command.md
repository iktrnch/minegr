---
type: feature
status: unimplemented
created: 2026-08-24
related_code: []
related_docs: []
---
# init-command

## Summary
`minegr init` - initialises the directory as a minecraft server.
### CLI Flags
| Flag                            | Description                                                                                                 | Mandatory |
| ------------------------------- | ----------------------------------------------------------------------------------------------------------- | --------- |
| `--name <name>`                 | Sets the server display name. Defaults to the current directory name when omitted.                          | No        |
| `--platform <platform>`         | Selects the server platform. Supported values: `vanilla`, `paper`, or `fabric`.                             | No        |
| `--minecraft-version <version>` | Selects the Minecraft version. The version must be supported by the chosen platform.                        | No        |
| `--memory <size>`               | Sets the maximum JVM heap size, such as `2GiB` or `4096MiB`. Uses a system-derived default when omitted.    | No        |
| `--port <port>`                 | Sets the Minecraft server port. Defaults to `25565`.                                                        | No        |
| `--accept-eula`                 | Confirms acceptance of the Minecraft EULA. When omitted, interactive initialization prompts for acceptance. | No        |
| `--yes`                         | Skips the final configuration confirmation. It does not imply EULA acceptance.                              | No        |

## Behaviour
Creates the `minegr.toml` unless already created
During the creation parameters can be passed as CLI flags or prompted in terminal. If the value is present in the CLI flag, the prompt is omitted. If the value in the CLI is invalid - print and error and exit.
## Workflow
### If `minegr.toml` is not found
1. Prompt for a name
2. Prompt for a server platform from
	- Vanilla
	- Paper
	- Fabric
3. Prompt for a minecraft version
4. Prompt for EULA agreement
	- Exit if user selects no
5. Prompt for memory limit
	- Default 2 GiB
6. Prompt for server port
	- Default 25565
	- Check whether the port is already occupied
	- Warn and ask for another value if it is unavailable
7. Show a summary and ask for confirmation
	- Exit if denied
8. Create `minegr.toml`
### If `minegr.toml` is found or just created
1. Run validation:
	- Determine the Java version required by that Minecraft version.
	- Find a compatible Java installation.
	- Check available memory and disk space.
	- Check port availability.
	- Check directory permissions.
	- Download or resolve the selected server software.
	- Report warnings and actionable errors.
2. Print the next action:
```
Configuration: ./minegr.toml
Start it with: minegr start
```
## Failure cases
- Stdin is not terminal and user prompts are required
## Implementation
- Use `clap` to parse CLI arguments into a struct
- Instantiate a struct for init parameter
	- For each struct entry take from CLI struct if Some or run a prompt if None
- Create `minegr.toml`
- Run validation
- Print next action message