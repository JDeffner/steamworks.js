import * as steamworks from "../../index";

export default function main() {
	const client = steamworks.init(480);
	console.log(client.localplayer.getName())
	void client.workshop.deleteItem(1n)

	client.input.init()
	client.input.setInputActionManifestFilePath("./game_actions_480.vdf")
	client.input.runFrame()

	const actionSet: bigint = client.input.getActionSet("InGameControls")
	const jump: bigint = client.input.getDigitalAction("jump")
	const move: bigint = client.input.getAnalogAction("move")

	for (const controller of client.input.getControllers()) {
		const jumpOrigins: number[] = controller.getDigitalActionOrigins(actionSet, jump)
		const moveOrigins: number[] = controller.getAnalogActionOrigins(actionSet, move)

		for (const origin of [...jumpOrigins, ...moveOrigins]) {
			const glyphPath: string = client.input.getGlyphForActionOrigin(origin)
			const label: string = client.input.getStringForActionOrigin(origin)
			console.log(`${label} -> ${glyphPath}`)
		}

		const opened: boolean = controller.showBindingPanel()
		if (!opened) {
			console.log("could not open the binding panel")
		}
	}

	client.input.shutdown()
}
