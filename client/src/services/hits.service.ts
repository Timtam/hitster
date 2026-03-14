import {
    FullHit,
    HitIssue,
    HitQueryPart,
    HitSearchQuery,
    Pack,
    PacksResponse,
    PaginatedHitsResponse,
} from "../entities"
import fetchAuth from "../fetch"

function buildUrl(path: string, params: URLSearchParams): string {
    const query = params.toString()
    return query ? `${path}?${query}` : path
}

function appendParam(
    params: URLSearchParams,
    key: string,
    value: string | number | undefined,
) {
    if (value === undefined) return
    params.append(key, String(value))
}

function appendArrayParam(
    params: URLSearchParams,
    key: string,
    values: readonly (string | number)[] | undefined,
) {
    values?.forEach((value) => {
        params.append(key, String(value))
    })
}

function appendPackParam(
    params: URLSearchParams,
    key: string,
    values: readonly string[] | undefined,
) {
    if (values === undefined) return
    if (values.length === 0) {
        params.append(key, "")
        return
    }
    appendArrayParam(params, key, values)
}

export default class HitService {
    async getAllPacks(): Promise<Pack[]> {
        const res = await fetch("/api/hits/packs", {
            method: "GET",
        })
        return PacksResponse.parse(await res.json()).packs
    }

    async searchHits(query: HitSearchQuery): Promise<PaginatedHitsResponse> {
        const params = new URLSearchParams()
        appendArrayParam(params, "sort_by", query.sort_by)
        appendParam(params, "sort_direction", query.sort_direction)
        appendParam(params, "query", query.query)
        appendPackParam(params, "packs", query.packs)
        appendParam(params, "start", query.start)
        appendParam(params, "amount", query.amount)
        appendArrayParam(params, "parts", query.parts)
        appendArrayParam(params, "filters", query.filters)

        const res = await fetch(buildUrl("/api/hits/search", params), {
            method: "GET",
        })
        return PaginatedHitsResponse.parse(await res.json())
    }

    async get(
        id: string,
        parts?: HitQueryPart[],
    ): Promise<FullHit | undefined> {
        const params = new URLSearchParams()
        appendArrayParam(params, "parts", parts)

        const res = await fetch(buildUrl(`/api/hits/${id}`, params), {
            method: "GET",
        })

        if (res.status === 200) return FullHit.parse(await res.json())
    }

    async updateHit(hit: FullHit) {
        const res = await fetchAuth(`/api/hits/${hit.id}`, {
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

    async deleteHit(hit_id: string) {
        const res = await fetchAuth(`/api/hits/${hit_id}`, {
            method: "DELETE",
            credentials: "include",
        })

        if (res.status == 200) return
        throw { message: (await res.json()).message, status: res.status }
    }

    async deletePack(pack_id: string) {
        const res = await fetchAuth(`/api/hits/packs/${pack_id}`, {
            method: "DELETE",
            credentials: "include",
        })

        if (res.status == 200) return
        throw { message: (await res.json()).message, status: res.status }
    }

    async createPack(name: string): Promise<Pack> {
        const res = await fetchAuth(`/api/hits/packs`, {
            body: JSON.stringify({
                name: name,
            }),
            headers: {
                "Content-Type": "application/json",
            },
            method: "POST",
            credentials: "include",
        })

        if (res.status === 200) return Pack.parse(await res.json())
        throw { message: (await res.json()).message, status: res.status }
    }

    async updatePack(id: string, name: string) {
        const res = await fetchAuth(`/api/hits/packs/${id}`, {
            body: JSON.stringify({
                name: name,
            }),
            headers: {
                "Content-Type": "application/json",
            },
            method: "PATCH",
            credentials: "include",
        })

        if (res.status != 200)
            throw { message: (await res.json()).message, status: res.status }
    }

    async createHit(hit: FullHit) {
        const res = await fetchAuth(`/api/hits`, {
            body: JSON.stringify(hit),
            headers: {
                "Content-Type": "application/json",
            },
            method: "POST",
            credentials: "include",
        })

        if (res.status == 200) return FullHit.parse(await res.json())
        throw { message: (await res.json()).message, status: res.status }
    }

    async exportHits(query?: string, packs?: string[]): Promise<string> {
        const params = new URLSearchParams()
        appendParam(params, "query", query || undefined)
        appendPackParam(params, "pack", packs)

        const res = await fetchAuth(buildUrl("/api/hits/export", params), {
            method: "GET",
            credentials: "include",
        })

        if (res.status == 200) return await res.text()
        throw { message: (await res.json()).message, status: res.status }
    }

    async createIssue(
        hitId: string,
        message: string,
        altchaToken: string,
    ): Promise<HitIssue> {
        const res = await fetchAuth(`/api/hits/${hitId}/issues`, {
            body: JSON.stringify({ message, altcha_token: altchaToken }),
            headers: {
                "Content-Type": "application/json",
            },
            method: "POST",
            credentials: "include",
        })

        if (res.status === 200) return HitIssue.parse(await res.json())
        throw { message: (await res.json()).message, status: res.status }
    }

    async deleteIssue(hitId: string, issueId: string) {
        const res = await fetchAuth(`/api/hits/${hitId}/issues/${issueId}`, {
            method: "DELETE",
            credentials: "include",
        })

        if (res.status === 200) return
        throw { message: (await res.json()).message, status: res.status }
    }
}
