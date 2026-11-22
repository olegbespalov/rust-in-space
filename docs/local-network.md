# Local network co-op

Rust in Space supports campaigns for up to four pilots over an IPv4 local network. One player hosts the session; the host runs the simulation and shares game state with connected players. Both computers need compatible game versions.

## Start a session

1. Connect all computers to the same local network.
2. On the host computer, choose **Local Network** from the main menu, then **Host game**.
3. On each guest computer, open **Local Network** and select the host under **Available games**. If discovery does not show the host, enter one of the IPv4 addresses displayed in the host lobby.
4. Each player picks a ship and each guest marks **Ready**. The host chooses the difficulty and starts the campaign once at least one guest has joined and all guests are ready.

The game uses TCP port `41616` for the session and UDP port `41616` for lobby discovery. Allow the game through the host computer's firewall on private networks. Discovery relies on broadcast, which some routers and guest Wi-Fi networks block; direct IPv4 entry can work when discovery is blocked, provided the computers can reach each other.

## How a co-op campaign works

- The host selects the campaign difficulty and advances missions.
- Pilots share mission progress and score. Each player spends their own salvage in the Mercenary Market. When guests finish shopping, they mark themselves ready so the host can advance to the next briefing.
- Missions with multiple pilots have more asteroids and count-based objectives, 25% more frequent regular enemy spawns, and 50% more Dreadnought health. In Nebula, the spawn warning is shorter to maintain the increased spawn rate.
- The camera follows each player's ship across the 1920 × 1080 arena. The minimap shows teammates, enemies, and asteroids. The roster below it shows each teammate's current and maximum hull HP.
- When destroyed, a pilot spectates a surviving teammate until the next mission. **Esc** opens that pilot's menu while the battle continues.
- A guest who disconnects can rejoin from the host lobby between campaigns. If only the host remains, the next mission uses single-pilot settings. LAN scores stay in the session.

## Troubleshooting

- **The host is missing from Available games:** Confirm both computers are on a network that allows device-to-device traffic. Try the host's IPv4 address from the lobby instead; broadcast discovery may be disabled on the network.
- **A guest cannot connect by address:** Check that the address is current, both computers can communicate on the local network, and the host firewall allows TCP port `41616`.
- **The lobby works but discovery does not:** Check that UDP port `41616` is allowed for discovery. Direct address entry avoids relying on broadcast discovery.
- **The game reports that TCP port `41616` is in use:** Close the other application or game instance using that port before hosting.
- **A guest cannot join or rejoin:** Make sure the host is in the lobby and both players run compatible game versions. Joining and rejoining are available from the host lobby between campaigns.
