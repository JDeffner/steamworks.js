import * as steamworks from "../../index";

export default function main() {
	const client = steamworks.init(480);
	console.log(client.localplayer.getName())
	void client.workshop.deleteItem(1n)

	// Update an item with metadata, key/value tags and content descriptors.
	void client.workshop.updateItem(1n, {
		title: "My Item",
		description: "An item with structured metadata",
		changeNote: "Add key/value tags",
		tags: ["weapons", "balance"],
		allowAdminTags: false,
		metadata: JSON.stringify({ schema: 2, requires: ["core"] }),
		removeAllKeyValueTags: true,
		keyValueTags: [
			{ key: "category", value: "weapon" },
			{ key: "difficulty", value: "hard" }
		],
		removeKeyValueTags: ["deprecated"],
		contentDescriptors: [client.workshop.ContentDescriptor.FrequentViolenceOrGore],
		removeContentDescriptors: [client.workshop.ContentDescriptor.AnyMatureContent]
	})

	// Query items filtered by key/value tags, asking for tags and metadata back.
	void client.workshop.getAllItems(
		0,
		client.workshop.UGCQueryType.RankedByPublicationDate,
		client.workshop.UGCType.Items,
		480,
		480,
		{
			includeKeyValueTags: true,
			includeMetadata: true,
			cloudFileNameFilter: "level.dat",
			requiredKeyValueTags: [{ key: "category", value: "weapon" }]
		}
	).then(result => {
		for (const item of result.items) {
			if (!item) continue

			const metadata: string | undefined = item.metadata ?? undefined
			console.log(item.publishedFileId, metadata)

			for (const tag of item.keyValueTags ?? []) {
				console.log(`${tag.key} = ${tag.value}`)
			}
		}
	})

	// Pause workshop downloads while loading, then resume.
	client.workshop.suspendDownloads(true)
	client.workshop.suspendDownloads(false)
}
