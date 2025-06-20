import { PacksResponse } from "../entities"

export default class Hitservice {
    async getAllPacks(): Promise<Record<string, number>> {
        const res = await fetch("/api/hits/packs", {
            method: "GET",
        })
        return PacksResponse.parse(await res.json()).packs
    }
}
