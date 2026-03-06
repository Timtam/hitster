let userAuthRequest: Promise<void> | null = null

export async function refreshUserAuth(): Promise<void> {
    if (userAuthRequest === null) {
        userAuthRequest = fetch("/api/users/auth", {
            credentials: "include",
        })
            .then(() => undefined)
            .finally(() => {
                userAuthRequest = null
            })
    }

    await userAuthRequest
}
