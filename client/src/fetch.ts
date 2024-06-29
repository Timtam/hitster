export default async function fetchAuth(
    url: string,
    options?: Parameters<typeof fetch>[1],
): Promise<Response> {
    const res = await fetch(url, options)

    if (res.status === 401) {
        await fetch("/api/users/auth", {
            credentials: "include",
        })
        return await fetchAuth(url, options)
    }
    return res
}
