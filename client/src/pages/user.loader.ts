import type { LoaderFunction } from "react-router"
import { PublicUser, UserStats } from "../entities"
import UserService from "../services/users.service"

const loader: LoaderFunction = async ({
    params,
}): Promise<[PublicUser, UserStats]> => {
    if (params.userId === undefined)
        throw { message: "internal api error", status: 500 }

    const us = new UserService()
    const user = await us.get(params.userId)
    if (user === undefined) throw { message: "user id not found", status: 404 }

    const stats = await us.getStats(params.userId)
    return [user, stats]
}

export default loader
