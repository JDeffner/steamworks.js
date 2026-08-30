import * as steamworks from "../../index";
import type { matchmaking } from "../../client";

export default function main() {
	const client = steamworks.init(480);
	console.log(client.localplayer.getName())
	void client.workshop.deleteItem(1n)

	void (async () => {
		const existing = await client.leaderboard.findLeaderboard("Feet Traveled")
		if (existing === null) {
			console.log("no such leaderboard")
			return
		}

		const board = await client.leaderboard.findOrCreateLeaderboard(
			"Quickest Flag Capture",
			client.leaderboard.LeaderboardSortMethod.Ascending,
			client.leaderboard.LeaderboardDisplayType.TimeMilliSeconds
		)

		const handle: bigint = board.handle
		console.log(board.getName(), board.getEntryCount(), handle)

		const sortMethod = board.getSortMethod()
		if (sortMethod === client.leaderboard.LeaderboardSortMethod.Descending) {
			console.log("higher is better")
		}
		console.log(board.getDisplayType())

		const uploaded = await board.uploadScore(
			12345,
			client.leaderboard.LeaderboardUploadScoreMethod.KeepBest,
			[1, 2, 3]
		)
		if (uploaded.scoreChanged) {
			console.log(`rank ${uploaded.globalRankPrevious} -> ${uploaded.globalRankNew}`)
		}
		await board.uploadScore(1, client.leaderboard.LeaderboardUploadScoreMethod.ForceUpdate)

		const entries = await board.downloadEntries(
			client.leaderboard.LeaderboardDataRequest.GlobalAroundUser,
			-4,
			5,
			3
		)
		for (const entry of entries) {
			const steamId64: bigint = entry.steamId.steamId64
			console.log(entry.globalRank, entry.score, entry.details, steamId64)
		}

		await existing.downloadEntries(client.leaderboard.LeaderboardDataRequest.Friends, 0, 0, 0)
	})()

	client.stats.setFloat("distance_traveled", 12.5)
	const distance: number | null = client.stats.getFloat("distance_traveled")
	console.log(distance, client.stats.store())

	const friends = client.friends.getFriends(client.friends.FriendFlags.Immediate)
	for (const friend of friends) {
		const steamId = friend.getSteamId()
		const steamId64: bigint = steamId.steamId64
		const nickName: string | null = friend.getNickName()
		console.log(steamId64, steamId.steamId32, friend.getName(), nickName ?? "", friend.getState())

		const game = friend.getGamePlayed()
		if (game !== null) {
			console.log(game.appId, game.gameId, game.gameAddress, game.gamePort, game.queryPort, game.lobbyId)
		}

		// 32x32 RGBA
		const avatar: Buffer | null = friend.smallAvatar()
		if (avatar !== null) {
			console.log(avatar.length === 32 * 32 * 4)
		}
		console.log(friend.hasFriend(client.friends.FriendFlags.OnGameServer))
	}

	for (const coplayer of client.friends.getCoplayFriends()) {
		console.log(coplayer.getName(), coplayer.mediumAvatar()?.length, coplayer.largeAvatar()?.length)
	}

	const other = client.friends.getFriend(76561197960287930n)
	console.log(other.getName())
	const fetching: boolean = client.friends.requestUserInformation(76561197960287930n, false)
	console.log(fetching)

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
