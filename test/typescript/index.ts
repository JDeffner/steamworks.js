import * as steamworks from "../../index";

export default function main() {
	const client = steamworks.init(480);
	console.log(client.localplayer.getName())
	void client.workshop.deleteItem(1n)

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
}
