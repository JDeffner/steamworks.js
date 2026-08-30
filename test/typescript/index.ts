import * as steamworks from "../../index";
import type { matchmaking } from "../../client";

export default function main() {
	const client = steamworks.init(480);
	console.log(client.localplayer.getName())
	void client.workshop.deleteItem(1n)

	// Unfiltered lobby list, exactly as before filters existed
	void client.matchmaking.getLobbies().then(lobbies => {
		for (const lobby of lobbies) {
			console.log(lobby.id, lobby.getMemberCount())
		}
	})

	const filter: matchmaking.LobbyListFilter = {
		stringFilters: [{
			key: "gamemode",
			value: "ffa",
			comparison: client.matchmaking.LobbyStringComparison.Equal
		}],
		numberFilters: [{
			key: "elo",
			value: 1500,
			comparison: client.matchmaking.LobbyNumberComparison.GreaterThanEqualTo
		}],
		nearValueFilters: [{ key: "elo", value: 1800 }],
		slotsAvailable: 2,
		distance: client.matchmaking.LobbyDistanceFilter.Far,
		count: 20
	}

	void client.matchmaking.getLobbies(filter).then(async lobbies => {
		const lobby: matchmaking.Lobby | undefined = lobbies[0]
		if (lobby !== undefined) {
			const joined = await lobby.join()
			console.log(joined.getData("gamemode"))
			joined.leave()
		}
	})
}
