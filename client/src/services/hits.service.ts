import queryString from "query-string"
import {
    HitSearchQuery,
    Pack,
    PacksResponse,
    PaginatedHitsResponse,
} from "../entities"

export default class HitService {
    async getAllPacks(): Promise<Pack[]> {
        const res = await fetch("/api/hits/packs", {
            method: "GET",
        })
        return PacksResponse.parse(await res.json()).packs
    }

    async searchHits(query: HitSearchQuery): Promise<PaginatedHitsResponse> {
        const res = await fetch(
            "/api/hits/search?" + queryString.stringify(query),
            {
                method: "GET",
            },
        )
        return PaginatedHitsResponse.parse(await res.json())
    }
}
