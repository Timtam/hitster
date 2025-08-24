import type { LoaderFunction } from "react-router"
import { FullHit, Pack } from "../entities"
import HitService from "../services/hits.service"

const loader: LoaderFunction = async ({
    params,
}): Promise<[FullHit, Pack[]]> => {
    const hs = new HitService()

    if (params.hitId !== undefined) {
        const hit = await hs.get(params.hitId)

        if (hit === undefined)
            throw { message: "hit id not found", status: 404 }

        const packs = await hs.getAllPacks()

        return [hit, packs]
    }
    throw { message: "internal api error", status: 500 }
}

export default loader
