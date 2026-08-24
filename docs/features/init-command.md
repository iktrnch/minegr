---
type: feature
status: unimplemented
created: 2026-08-24
related_code: []
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
| `--uuid`                        | Regenerates the uuid of the `minegr.toml`                                                                   | No        |

## Behaviour
Creates the `minegr.toml` unless already created
During the creation parameters can be passed as CLI flags or prompted in terminal. If the value is present in the CLI flag, the prompt is omitted. If the value in the CLI is invalid - print and error and exit.
## Workflow
### If `--uuid` is passed
1. Create a new uuid for the `minegr.toml`
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
### Creating config
- Use `clap` to parse CLI arguments into a struct
- Instantiate a struct for init parameter
	- For each struct entry take from CLI struct if Some or run a prompt if None
- Create `minegr.toml`
- Run validation
- Print next action message
### Recreating uuid
- Use `uuid` crate to generate the uuid for the config

### Pulling minecraft core jar
**Vanilla**
- Get the version json from [Java Manifest for vanilla servers](https://piston-meta.mojang.com/mc/game/version_manifest_v2.json) under and its url from `url` json field
- Get the server download url from `downloads.server.url`
- Download the server

**Paper**
- Get available Minecraft versions from the [Paper Downloads API](https://fill.papermc.io/v3/projects/paper) under `versions`
- Get builds for the selected version from `https://fill.papermc.io/v3/projects/paper/versions/{minecraft_version}/builds`
- Select the newest build whose `channel` is `STABLE`
- Get the server download URL from `downloads["server:default"].url`
- Download the server JAR
- Include a descriptive `User-Agent` containing the application name, version, and contact URL or email in every API request, as [required by PaperMC](https://docs.papermc.io/misc/downloads-service/)

**Fabric**
- Get supported Minecraft versions from the [Fabric Meta API](https://meta.fabricmc.net/v2/versions/game)
- Get compatible Fabric Loader versions from `https://meta.fabricmc.net/v2/versions/loader/{minecraft_version}`
- Select the newest stable loader from `loader.version`
- Get available Fabric Installer versions from `https://meta.fabricmc.net/v2/versions/installer`
- Select the newest stable installer from `version`
- Construct the server launcher URL as `https://meta.fabricmc.net/v2/versions/loader/{minecraft_version}/{loader_version}/{installer_version}/server/jar`
- Download the Fabric server launcher JAR

Fabric’s downloaded JAR is a launcher: on first run, it downloads the required Minecraft server and Fabric files. It can still be treated as the server’s main executable JAR. See the [Fabric server installation documentation](https://wiki.fabricmc.net/install#server_simple_method).