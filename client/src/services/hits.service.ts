import { HitsStatus, PacksResponse } from "../entities"

export default class HitService {
    async getAllPacks(): Promise<Record<string, number>> {
        const res = await fetch("/api/hits/packs", {
            method: "GET",
        })
        return PacksResponse.parse(await res.json()).packs
    }

    async getStatus(): Promise<HitsStatus> {
        const res = await fetch("/api/hits/status", {
            method: "GET",
        })
        return HitsStatus.parse(await res.json())
    }
}
