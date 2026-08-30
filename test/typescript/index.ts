import * as steamworks from "../../index";

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
}
