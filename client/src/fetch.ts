import { refreshUserAuth } from "./user-auth"

export default async function fetchAuth(
    url: string,
    options?: Parameters<typeof fetch>[1],
): Promise<Response> {
    const res = await fetch(url, options)

    if (res.status === 401) {
        await refreshUserAuth()
        return await fetchAuth(url, options)
    }
    return res
}
