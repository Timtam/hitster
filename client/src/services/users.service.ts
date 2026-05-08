import { PublicUser, UserStats } from "../entities"

export default class UserService {
    async get(id: string): Promise<PublicUser | undefined> {
        const res = await fetch(`/api/users/${id}`, {
            method: "GET",
        })

        if (res.status === 200) return PublicUser.parse(await res.json())
    }

    async getStats(id: string): Promise<UserStats> {
        const res = await fetch(`/api/users/${id}/stats`, {
            method: "GET",
        })
        return UserStats.parse(await res.json())
    }
}
