import queryString from "query-string"
import {
    FullHit,
    HitSearchQuery,
    Pack,
    PacksResponse,
    PaginatedHitsResponse,
} from "../entities"
import fetchAuth from "../fetch"

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

    async get(id: string): Promise<FullHit | undefined> {
        const res = await fetch(`/api/hits/${id}`, {
            method: "GET",
        })

        if (res.status === 200) return FullHit.parse(await res.json())
    }

    async updateHit(hit: FullHit) {
        const res = await fetchAuth("/api/hits", {
            body: JSON.stringify(hit),
            headers: {
                "Content-Type": "application/json",
            },
            method: "PATCH",
            credentials: "include",
        })

        if (res.status == 200) return
        throw { message: (await res.json()).message, status: res.status }
    }
}
