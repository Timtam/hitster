import { Pack, PacksResponse } from "../entities"

export default class HitService {
    async getAllPacks(): Promise<Pack[]> {
        const res = await fetch("/api/hits/packs", {
            method: "GET",
        })
        return PacksResponse.parse(await res.json()).packs
    }
}
